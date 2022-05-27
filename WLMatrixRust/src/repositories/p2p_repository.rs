use std::sync::{Arc, Mutex};

pub struct P2PRepository {
    seq_number : Arc<Mutex<Vec<u32>>> 
}

impl P2PRepository {


    pub fn new() -> Self {
        return P2PRepository{seq_number: Arc::new(Mutex::new(Vec::new()))};
    }

    pub fn set_seq_number(&self, seq_number: u32) {
        let mut seq_number_mtx =  self.seq_number.lock().expect("Could not lock mutex");
        seq_number_mtx.push(seq_number);
    }

    pub fn get_seq_number(&self) -> u32 {
        return self.seq_number.lock().expect("Could not lock mutex").last().unwrap().to_owned();
    }

}