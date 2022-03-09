use nacl::sign::verify;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
struct VerificationRequest {
   payload: String,
   signature: String,
   transaction_id: Option<String>,
   pub_key: Option<String>,
}

#[derive(Serialize)]
struct VerificationResponse {
    transaction_id: String,
    complete: bool,
    valid: bool,
}

impl VerificationRequest {
    pub fn new() -> Self {
        VerificationRequest {
            payload: "".to_owned(),
            signature: "".to_owned(),
            transaction_id: None,
            pub_key: None,
        }
    }
}

fn main() {
    dotenv::dotenv().ok();
    let svc_name = std::env::var("SVC_NAME").expect("SVC_NAME");
    let stack_name = format!("stack:{}:verification_request_dlq", svc_name);

    // The service could fail at either one of these calls, but I think that's acceptable.  If
    // there's an error with the connection, the application can't function, so we fail early here
    // and let errors propagate out.
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut connection = client.get_connection().unwrap();

    // This will block indefinitely until a new message is written to "queue:verification_request".
    // The service should run indefinitely, reading subsequent writes to that List.
    //
    // The BRPOPLPUSH command here will block until a message is available on the
    // "queue:verification_request" list.  New messages will be popped from the right of the list,
    // and then written to the left of the "stack:<svc_name>:verification_request_dlq" list.  The latter list
    // is an intermediate list used as a stack that will prevent us from losing requests should the service goes
    // down.
    //
    // This both allows us to make sure that no requests are lost, but also allows us, using the
    // SVC_NAME env var to scale the service horizontally.  Each service processing messages can
    // have it's own stack, which is can check on startup, and which can be monitored subsequently
    // for dead messages.
    //
    // We might also flush any messages left in the stack to another unified DLQ when the service
    // exits.  This way, assuming the service exits gracefully, we get some collection of dead
    // messages in a central place that can be acted on appropriately.
    //
    // Also, at any point during this process, where we reach an error state, we may want to
    // re-queue the mesasge at the back of the "queue:verification_request" queue to be processed
    // again.
    while let verification_request = redis::cmd("BRPOPLPUSH").arg("queue:verification_request").arg(&stack_name).arg("0").query(&mut connection) {
        // Retrieve string from Result, or default to empty string
        let verification_request = verification_request.unwrap_or("".to_owned());
        // Attempt to deserialize string, or initialize a placeholder object
        let verification_request = serde_json::from_str(&verification_request).unwrap_or(VerificationRequest::new());
        // Check that we got the required values to complete verification.  If the Options in the
        // below two fields are none, then either we had an issue when pulling the value from
        // Redis, or we had an issue deserializing.
        //
        // In production code, we'd want to do some logging here to make sure that we had
        // visibility into this.  For the sake of time, I haven't done that here.
        if verification_request.pub_key.is_none() || verification_request.transaction_id.is_none() { continue; }

        // Extract the pertinent values from the struct
        let signature = verification_request.signature.clone();
        let payload = verification_request.payload.clone();
        let transaction_id = verification_request.transaction_id.unwrap().clone();
        let pub_key = verification_request.pub_key.unwrap().clone();

        // Use NaCL implementation in Rust to verify the message using the signature and the public
        // key.  Here if we have an error when verifying the signature, we count that as a false as
        // well.  In production, depending on the purpose and use-case, we may want to treat this
        // differently and requeue, or possibly provide out a meaningful error to the user.
        let valid = verify(signature.as_bytes(), payload.as_bytes(), pub_key.as_bytes()).unwrap_or(false);

        // Prepare the final payload to return to the account service
        let response = VerificationResponse {
            complete: true,
            transaction_id: transaction_id.clone(),
            valid,
        };
        // Serialize
        let payload = serde_json::to_string(&response);
        if let Ok(payload) = payload {
            // Write to the completed request queue
            let write_key: redis::RedisResult<()> = redis::cmd("SET").arg(transaction_id).arg(payload).query(&mut connection);
            // If everything succeeded, we pull the value off of the DLQ.
            if write_key.is_ok() {
                let pop_stack: redis::RedisResult<()> = redis::cmd("LPOP").arg(&stack_name).query(&mut connection);
            }
        }
    }
}
