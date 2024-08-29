use core::str;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Receiver, Sender};
use clap::Parser;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::{task, net::TcpListener, net::TcpStream};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8000, help = "port for running the node")]
    port: u32,

    #[arg(help = "list of peers to connect to")]
    peers: Vec<String>,
}

const SIZE: usize = 50;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let peers = args.peers;
    let (tx, rx) = mpsc::channel::<String>(100);

    for peer in peers {
        tx.send(peer).await.expect("Failed to add the peers");
    }

    task::spawn(async move {
        client_reader(rx, format!("127.0.0.1:{}", args.port)).await;
    });
    server(args.port, tx).await;
}

async fn client_reader(mut rx: Receiver<String>, server_addr: String) {
    while let Some(addr) = rx.recv().await {
        let server_addr = server_addr.clone();
        task::spawn(async move {
            println!("Connecting to {addr}");
            let mut stream :Option<TcpStream> = None;
            while stream.is_none() {
                match TcpStream::connect(&addr).await {
                    Ok(mut s) => {
                        println!("Successfully connected to the {addr:?}");
                        let mut msg = server_addr.trim().as_bytes().to_vec();
                        msg.resize(SIZE, 0);
                        s.write_all(&mut msg).await.unwrap();
                        stream = Some(s)
                    }
                    Err(_) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                };
            }
            let mut stream = stream.unwrap();
            loop {
                let mut buff = vec![0 as u8; SIZE];
                match stream.read_exact(&mut buff).await {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        println!("{}",str::from_utf8(&msg).unwrap());
                    },
                    Err(_) => {eprintln!("Connection closed");return}
                }
            }
        });
    }
}

async fn server(port: u32, tx: Sender<String>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}",port)).await.unwrap();
    println!("Server is listening on port {}...",port);

    let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));
    let clients_clone = clients.clone();

    task::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    println!("Client connected: {addr}");
                    let mut buff = vec![0 as u8; SIZE];
                    match stream.read_exact(&mut buff).await {
                        Ok(_) => {
                            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                            let msg = str::from_utf8(&msg).unwrap();
                            println!("peer {} sented node address {}",stream.peer_addr().unwrap(), msg);
                            tx.send(msg.to_string()).await.expect("Failed to add the peer");
                        },
                        Err(_) => {eprintln!("Connection closed");return}
                    }
                    clients_clone.lock().unwrap().push(stream);
                }
                Err(e) => eprintln!("Failed to accept the connection: {e}")
            }
        }
    });
    write_to_clients(clients).await;
}

async fn write_to_clients(clients: Arc<Mutex<Vec<TcpStream>>>) {
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    while let Ok(Some(msg)) = lines.next_line().await {

        let mut msg = msg.trim().as_bytes().to_vec();
        msg.resize(SIZE, 0);

        let mut clients = clients.lock().unwrap();
        let mut clients_to_remove = vec![];

        for (i, client) in clients.iter_mut().enumerate() {
            if let Err(_) = client.write_all(&mut msg).await {
                clients_to_remove.push(i)
            }
        }

        for &index in clients_to_remove.iter().rev() {
            clients.remove(index);
        }
    }
}