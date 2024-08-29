

use std::mem;

use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct Node {
    rx: Option<Receiver<String>>, 
    tx: Sender<String>
}

impl Node {
    pub fn new_node() -> Node {
        let (tx, rx) = mpsc::channel::<String>(30);
        return Node{
            rx: Some(rx),
            tx
        };
    }

    pub fn server_start(&mut self) {

    }

    pub fn client_start(&mut self) {

    }

    pub fn add_peer(&mut self, add: &str) {
    }

    pub fn take_reciever(&mut self) -> Option<Receiver<String>> {
        mem::take(&mut self.rx)
    }

    pub fn send_msg(&mut self, msg: &str) {

    }
}