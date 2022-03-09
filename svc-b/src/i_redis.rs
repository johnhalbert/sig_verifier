pub struct Redis {
    _client: redis::Client,
    connection: redis::Connection,
}

impl Redis {
    pub fn new() -> redis::RedisResult<Self> {
        let _client = redis::Client::open("redis://127.0.0.1/")?;
        let connection = _client.get_connection()?;

        Ok(Redis { _client, connection })
    }

    pub fn push_verification_request(&mut self, verification_request: &crate::account::VerificationRequest) -> Result<(), ()> {
        let payload = serde_json::to_string(verification_request).map_err(|_| ())?;
        redis::cmd("RPUSH").arg("queue:verification_request").arg(payload).query(&mut self.connection).map_err(|_| ())?;
        Ok(())
    }
    
    pub fn upsert_account(&mut self, account_id: String, pub_key: String) -> Result<(), ()> {
        redis::cmd("SET").arg(account_id).arg(pub_key).query(&mut self.connection).map_err(|_| ())?;
        Ok(())
    }

    pub fn upsert_verification_request(&mut self, transaction_id: String, state: crate::account::VerificationResponse) -> Result<(), ()> {
        let payload = serde_json::to_string(&state).map_err(|_| ())?;
        redis::cmd("SET").arg(transaction_id).arg(payload).query(&mut self.connection).map_err(|_| ())?;
        Ok(())
    }

    pub fn get_verification_status(&mut self, transaction_id: String) -> Result<String, ()> {
        redis::cmd("GET").arg(transaction_id).query(&mut self.connection).map(|v: String| Ok(v)).map_err(|_| ())?
    }
}
