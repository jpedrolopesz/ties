

pub struct Message {
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
}

impl Message {

    pub fn create(sender: String, text: String) -> Message {
        Message{
            text,
            sender,
            timestamp: current_timestamp()
        }
    }

    pub fn print(&self) {
        println!("At {}: {} says: {}",self.timestamp, self.sender, self.timestamp)
    }
    
}