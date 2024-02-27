pub struct Session {
    pub session_key: Option<Vec<u8>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            session_key: None,
        }
    }
}