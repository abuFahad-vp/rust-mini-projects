mod peer_network;

use tokio;
use peer_network::Node;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {

    let mut node = Node::new_node();
    node.add_peer("127.0.0.1:8001");
    node.server_start();
    node.client_start();

    let msg_reciever = node.take_reciever();

    tokio::spawn(async move {
        if let Some(mut reciever) = msg_reciever {
            while let Some(msg) = reciever.recv().await {
                println!("{msg}")
            }
        }
    });

    loop {
        let stdin = BufReader::new(io::stdin());
        let mut lines = stdin.lines();
        while let Ok(Some(msg)) = lines.next_line().await {
            node.send_msg(msg.trim());
        }
    }
}