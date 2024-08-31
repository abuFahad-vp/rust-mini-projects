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

    let mut node = peer_network::Node::new(args.port).await;

    node.server_listen().await;

    for addr in args.peers {
        node.add_peer(addr).await;
    }

    let msg_reciever = node.take_reciever();
    let msg_transmitter = node.take_sender();

    tokio::spawn(async move {
        if let Some(mut reciever) = msg_reciever {
            while let Some(msg) = reciever.recv().await {
                println!("{msg}");
                if let Err(e) = msg_transmitter.send(msg).await {
                    eprintln!("Cannot transmit the message to internal reciever: {e}")
                }
            }
        }
    });

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let msg_sender = node.take_sender();
    while let Ok(Some(msg)) = lines.next_line().await {
        let msg = msg.trim();
        let msg: Vec<&str> = msg.split(":").collect();
        if msg[0] == "peer" {
            node.add_peer(msg[1].to_string()).await;
        } else {
            if let Err(e) = msg_sender.send(msg[0].to_string()).await {
                eprintln!("Cannot transmit the message to internal reciever: {e}")
            }
        }
    }
}
