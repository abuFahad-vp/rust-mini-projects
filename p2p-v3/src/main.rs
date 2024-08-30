mod peer_network;

use clap::Parser;

use tokio::io::{AsyncBufReadExt, BufReader}; 
/*
HNDSHK -> handshake
01 -> Requesting to accept the handshake
10 -> handshake accepted
00 -> Handshake rejected
 */

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8000, help = "port for running the node")]
    port: u32,

    #[arg(help = "list of peers to connect to")]
    peers: Vec<String>,
}

#[tokio::main]
async fn main() {

    let args = Args::parse();

    let mut node = peer_network::Node::new(args.port);

    node.server_listen().await;

    for addr in args.peers {
        node.add_peer(addr).await;
    }

    let msg_reciever = node.take_reciever();

    tokio::spawn(async move {
        if let Some(mut reciever) = msg_reciever {
            while let Some(msg) = reciever.recv().await {
                println!("{msg}")
            }
        }
    });

    loop {
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();
        while let Ok(Some(msg)) = lines.next_line().await {
            node.send_msg(msg.trim()).await;
        }
    }
}
