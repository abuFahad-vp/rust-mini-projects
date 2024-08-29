use core::str;
use std::vec;
use std::{collections::HashSet, mem};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, broadcast,Mutex};

const SIZE: usize = 50;

pub struct Node {
    msg_client_tx: mpsc::Sender<String>,
    msg_client_rx: Option<mpsc::Receiver<String>>, 
    msg_server_tx: broadcast::Sender<String>,
    peer_tx: mpsc::Sender<String>,
    peer_rx: Option<mpsc::Receiver<String>>,
}

impl Node {
    pub fn new_node() -> Node {

        let (msg_client_tx, msg_client_rx) = mpsc::channel::<String>(100);
        let (peer_tx, peer_rx) = mpsc::channel::<String>(100);
        let (msg_server_tx, _) = broadcast::channel(100);

        return Node{
            msg_client_rx: Some(msg_client_rx),
            msg_client_tx,
            msg_server_tx,
            peer_rx: Some(peer_rx),
            peer_tx,
            // peers: Arc::new(Mutex::new(HashSet::new())),
        };
    }

    pub async fn server_listen(&mut self, port: u32) {
        let msg_server_rx = self.msg_server_tx.subscribe();
        let msg_client_tx = self.msg_client_tx.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("127.0.0.1:{}",port)).await.unwrap();
            println!("Server is listening on port {}...", port);

            let clients_connected = Arc::new(Mutex::new(Vec::<OwnedWriteHalf>::new()));
            let clients_connected_clone = clients_connected.clone();

            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, addr)) => {
                            let msg_client_tx = msg_client_tx.clone();
                            println!("Client connected: {addr}");
                            let (read_half, write_half) = stream.into_split();
                            tokio::spawn(async move {
                                Self::client_recv_msg(read_half, msg_client_tx).await;
                            });
                            clients_connected_clone.lock().await.push(write_half);
                        }
                        Err(e) => eprintln!("Failed to accept the connection: {e}")
                    }
                }
            });
            let client_connected_clone = clients_connected.clone();
            tokio::spawn(async move {
                Self::server_send_to_clients(msg_server_rx, client_connected_clone).await;
            });
        });
    }

    async fn server_send_to_clients(mut msg_server_rx: broadcast::Receiver<String>, clients: Arc<Mutex<Vec<OwnedWriteHalf>>>) {
        while let Ok(msg) = msg_server_rx.recv().await {
            let mut msg = msg.trim().as_bytes().to_vec();
            msg.resize(SIZE, 0);

            let mut clients = clients.lock().await;
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

    // here we're client
    pub async fn client_listen(&mut self) {

        let peer_rx= mem::take(&mut self.peer_rx);

        let msg_client_tx = self.msg_client_tx.clone();
        let msg_server_tx = self.msg_server_tx.clone();

        // connect to the peers
        tokio::spawn(async move {
            if let Some(mut peer_rx) = peer_rx {
                while let Some(addr) = peer_rx.recv().await {

                    let msg_client_tx = msg_client_tx.clone();
                    let msg_server_tx = msg_server_tx.clone();

                    tokio::spawn(async move {
                        println!("Connecting to {addr}");
                        let mut stream: Option<TcpStream> = None;
                        while stream.is_none() {
                            match TcpStream::connect(&addr).await {
                                Ok(s) => {
                                    println!("Successfully connected to {addr}");
                                    stream = Some(s)
                                }
                                Err(_) => {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                                }
                            }
                        }
                        let (read_half, write_half) = stream.unwrap().into_split();
                        tokio::spawn(async move {
                            Self::client_recv_msg(read_half, msg_client_tx).await;
                        });
                        Self::client_send_msg(write_half, msg_server_tx.subscribe()).await
                    });
                }
            }
        });
    }

    // send the message to peer servers which get from the sender of node
    async fn client_send_msg(mut stream: OwnedWriteHalf, mut msg_rx: broadcast::Receiver<String>) {
        while let Ok(msg) = msg_rx.recv().await {
            let mut msg = msg.trim().as_bytes().to_vec();
            msg.resize(SIZE, 0);
            if let Err(e) = stream.write_all(&mut msg).await {
                println!("Failed to send the message to client {}: {e}", stream.peer_addr().unwrap());
            }
        }
    }

    // transmit recieved msg to reciever of node
    async fn client_recv_msg(mut stream: OwnedReadHalf, msg_tx: mpsc::Sender<String>) {
        loop {
            let mut buff = vec![0 as u8; SIZE];
            match stream.read_exact(&mut buff).await {
                Ok(_) => {
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    if let Err(e) = msg_tx.send(String::from_utf8(msg).unwrap()).await {
                        println!("Failed to send the message from client {} to message reciever due to {e}", stream.peer_addr().unwrap())
                    }
                },

                Err(e) => {
                    eprint!("Connection closed: {e}"); 
                    return
                }
            }
        }
    }

    pub async fn add_peer(&mut self, addr: &str) {
        if let Err(e) = self.peer_tx.send(addr.to_string()).await {
            println!("Failed to add the peer {addr}: {e}")
        }
    }

    pub fn take_reciever(&mut self) -> Option<mpsc::Receiver<String>> {
        mem::take(&mut self.msg_client_rx)
    }

    pub fn send_msg(&mut self, msg: &str) {
        if let Err(e) = self.msg_server_tx.send(msg.to_string()) {
            println!("Failed to send the msg: {e}")
        }
    }
}