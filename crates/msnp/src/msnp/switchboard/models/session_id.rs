use std::fmt::Display;
use rand::Rng;

#[derive(Clone)]
pub struct SessionId(u16);

impl SessionId {
    
    pub fn new(value: u16) -> SessionId {
        SessionId(value)
    }
    
    pub fn empty() -> SessionId {
        SessionId(0)
    }
    
    pub fn random() -> SessionId {
        let mut rng = rand::thread_rng();
        let rand: u16 = rng.gen();
        SessionId(rand)
    }
}

impl From<u16> for SessionId {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

impl Into<u16> for SessionId {
    fn into(self) -> u16 {
        self.0
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}