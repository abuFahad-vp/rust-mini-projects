use uuid::Uuid;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::OwnedWriteHalf, net::tcp::OwnedReadHalf};
use std::{str::FromStr, sync::Arc};

const SIZE: usize = 50;

pub struct Node {
    node_id: Uuid,
    port: u32,

    write_streams: Arc<tokio::sync::Mutex<Vec<OwnedWriteHalf>>>,

    peer_connected: Arc<tokio::sync::Mutex<Vec<Uuid>>>,

    msg_incoming_tx: tokio::sync::mpsc::Sender<String>,
    msg_incoming_rx: Option<tokio::sync::mpsc::Receiver<String>>,
}

impl Node {
    pub fn new(port: u32) -> Node {

        let (msg_incoming_tx, msg_incoming_rx) = tokio::sync::mpsc::channel::<String>(100);

        Node {
            node_id: Uuid::new_v4(),
            port,

            write_streams: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            peer_connected: Arc::new(tokio::sync::Mutex::new(Vec::new())),

            msg_incoming_tx,
            msg_incoming_rx: Some(msg_incoming_rx),
        }
    }

    pub async fn server_listen(&self) -> tokio::task::JoinHandle<u32> {

        let port = self.port;
        let node_id = self.node_id.clone();

        let peer_connected = self.peer_connected.clone();
        let write_streams = self.write_streams.clone();
        let msg_incoming_tx = self.msg_incoming_tx.clone();

        tokio::spawn(async move {

            let node_id = node_id.clone();
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}",port)).await.expect("Failed to bind the socket to the address");

            println!("Server listening on port {}",port);

            loop {

                let peer_connected = peer_connected.clone();
                let write_streams = write_streams.clone();
                let msg_incoming_tx = msg_incoming_tx.clone();

                if let Ok((stream, addr)) = listener.accept().await {
                    println!("New Peer Connected: {addr}");
                    tokio::spawn(async move {
                        Self::handle_connection(stream, node_id, peer_connected, write_streams, msg_incoming_tx).await;
                    });
                }
            }
        })
    }

    async fn handle_connection(
        mut stream: tokio::net::TcpStream, 
        node_id: Uuid, 
        peer_connected: Arc<tokio::sync::Mutex<Vec<Uuid>>>,
        write_streams: Arc<tokio::sync::Mutex<Vec<OwnedWriteHalf>>>,
        msg_incoming_tx: tokio::sync::mpsc::Sender<String>,
    ) {
        
        let mut peer_request = vec![0 as u8; 46]; // for reading uuid of client

        if let Ok(_) = stream.read_exact(&mut peer_request).await {
            let response = String::from_utf8(peer_request).unwrap();
            let response: Vec<&str> = response.split(":").collect();
            match response[..] {
                // if proper request
                ["HNDSHK", "01", uuid] => {
                    let mut peer_connected = peer_connected.lock().await;
                    let peer_uuid = Uuid::from_str(uuid).unwrap();
                    if !peer_connected.contains(&peer_uuid) {

                        peer_connected.push(peer_uuid);

                        println!("Peer {uuid} accepted");

                        // respond with success message
                        let handshake_msg = format!("HNDSHK:10:{}", node_id);
                        stream.write_all(handshake_msg.as_bytes()).await.unwrap();

                        let (read_half, write_half) = stream.into_split();
                        let mut write_streams = write_streams.lock().await;

                        write_streams.push(write_half); // add to write_streams for writing to the clients

                        Self::peer_receiver(read_half, msg_incoming_tx).await;
                    }
                }
                _ => {
                    println!("Invalid response. Handshake rejected");
                    let handshake_msg = format!("HNDSHK:00:{}", node_id);
                    stream.write_all(handshake_msg.as_bytes()).await.unwrap();
                }
            }
        }

    }

    // Connecting to peer server. if successful use this thread to recieve messages and add the WriteHalf to collection for writing
    pub async fn add_peer(&self, addr: String) {

        let node_id = self.node_id;


        let peer_connected = self.peer_connected.clone();
        let write_streams = self.write_streams.clone();
        let msg_incoming_tx = self.msg_incoming_tx.clone();

        tokio::spawn(async move {
            println!("Connecting to {addr}...");
            let mut stream: Option<tokio::net::TcpStream> = None;
            while stream.is_none() {
                match tokio::net::TcpStream::connect(addr.clone()).await {
                    Ok(s) => {
                        println!("Successfully connected to {addr}. Waiting for handshake to complete");
                        stream = Some(s)
                    }
                    Err(_) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                }
            }

            let mut stream = stream.unwrap();
            let handshake_msg = format!("HNDSHK:01:{}",node_id);
            stream.write_all(handshake_msg.as_bytes()).await.unwrap();

            let mut peer_response = vec![0 as u8; 46];
            if let Ok(_) = stream.read_exact(&mut peer_response).await {
                let response = String::from_utf8(peer_response).unwrap();
                let response: Vec<&str> = response.split(":").collect();
                if response[0] == "HNDSHK" {
                    if response[1] == "10" {
                        let mut peer_connected = peer_connected.lock().await;
                        let peer_uuid = Uuid::from_str(response[2]).unwrap();
                        if !peer_connected.contains(&peer_uuid) {

                            peer_connected.push(peer_uuid);

                            println!("Peer {peer_uuid} accepted");

                            let (read_half, write_half) = stream.into_split();
                            let mut write_streams = write_streams.lock().await;

                            write_streams.push(write_half); // add to write_streams for writing to the clients

                            Self::peer_receiver(read_half, msg_incoming_tx).await;
                            return
                        }
                    }
                }
                println!("Handshake rejected");
            }
        });
    }

    async fn peer_receiver(mut stream: OwnedReadHalf, msg_incoming_tx: tokio::sync::mpsc::Sender<String>) {
        tokio::spawn(async move {
            loop {
                let mut buff = vec![0 as u8; SIZE];
                match stream.read_exact(&mut buff).await {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        if let Err(e) = msg_incoming_tx.send(String::from_utf8(msg).unwrap()).await {
                            println!("Failed to send the message from client {} to message reciever due to {e}", stream.peer_addr().unwrap())
                        }
                    },

                    Err(e) => {
                        eprintln!("Connection closed: {e}"); 
                        return
                    }
                }
            }
        });
    }

    pub fn take_reciever(&mut self) -> Option<tokio::sync::mpsc::Receiver<String>> {
        std::mem::take(&mut self.msg_incoming_rx)
    }

    pub async fn send_msg(&mut self, msg: &str) {

        let mut write_stream = self.write_streams.lock().await;

        let mut msg = msg.trim().as_bytes().to_vec();
        msg.resize(SIZE, 0);

        let mut clients_to_remove = vec![];

        for (i, stream) in write_stream.iter_mut().enumerate() {
            if let Err(e) = stream.write_all(&msg).await {
                println!("Failed to write to the peer {}: {e}",stream.peer_addr().unwrap());
                clients_to_remove.push(i)
            }
        }

        for &index in clients_to_remove.iter().rev() {
            write_stream.remove(index);
        } 
    }
}