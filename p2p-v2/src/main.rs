mod peer_network;

use tokio;
use clap::Parser;
use peer_network::Node;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8000, help = "port for running the node")]
    port: u32,

    #[arg(help = "list of peers to connect to")]
    peers: Vec<String>,
}

#[tokio::main]
async fn main() {

    let mut node = Node::new_node();

    let args = Args::parse();

    for addr in args.peers {
        node.add_peer(&addr).await;
    }

    node.client_listen().await;
    node.server_listen(args.port).await;

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