use uuid::Uuid;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}};
use core::str;
use std::{collections::HashMap, str::FromStr, sync::Arc};

use super::blockchain_core::{Block, Transaction};

#[derive(Debug)]
pub enum Protocol {
    Handshake, //handshake (permanent connection)
    Unknown
}

#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, Clone)]
pub struct Message {
    pub uuid: String,
    // pub msg_id: u32,
    pub block: Option<Block>, // msg_id 0
    pub transaction: Option<Transaction>, // msg_id 1
    pub message_hash: String,
}

#[derive(Clone)]
pub struct Node {
    pub node_id: Uuid,
    pub port: u16,

    pub write_streams: Arc<tokio::sync::Mutex<HashMap<Uuid, OwnedWriteHalf>>>,

    pub peer_connected: Arc<tokio::sync::Mutex<Vec<Uuid>>>,
    pub peer_server_addr: Arc<tokio::sync::Mutex<Vec<String>>>,

    pub msg_incoming_tx: crossbeam_channel::Sender<Message>,
    pub msg_incoming_rx: crossbeam_channel::Receiver<Message>,

    pub msg_outgoing_tx: crossbeam_channel::Sender<Message>,
    // msg_outgoing_rx: crossbeam_channel::Receiver<Message>,

    pub msg_hashes: Arc<tokio::sync::Mutex<HashMap<String, bool>>>,
    server_addr: String,
}

impl Node {
    pub async fn new(port: u16, server_addr: String) -> Node {

        let (msg_incoming_tx, msg_incoming_rx) = crossbeam_channel::unbounded();
        let (msg_outgoing_tx, msg_outgoing_rx) = crossbeam_channel::unbounded();

        let write_streams =  Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let msg_hashes = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let peer_server_addr = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let node_id = Uuid::new_v4();

        println!("Node id: {node_id}");

        Self::send_msg(msg_outgoing_rx.clone(), write_streams.clone(), msg_hashes.clone()).await;
        // Self::message_reciever(msg_incoming_rx.clone(), msg_outgoing_tx.clone()).await;

        Node {
            node_id,
            port,

            peer_connected: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            peer_server_addr,
            write_streams,

            msg_incoming_tx,
            msg_incoming_rx,

            msg_outgoing_tx,
            // msg_outgoing_rx,
            msg_hashes,
            server_addr
        }
    }

    // async fn message_reciever(
    //     msg_incoming_rx: crossbeam_channel::Receiver<Message>,
    //     msg_outgoing_tx: crossbeam_channel::Sender<Message>
    // ) {
    //     tokio::spawn(async move {
    //         let reciever = msg_incoming_rx;
    //         while let Ok(msg) = reciever.recv() {
    //             println!("{msg:?}");
    //             if let Err(e) = msg_outgoing_tx.send(msg) {
    //                 eprintln!("Cannot transmit the message to internal reciever: {e}")
    //             }
    //         }
    //     });
    // }


    pub fn get_id(&self) -> Uuid {
        self.node_id
    }

    pub async fn server_listen(&self) -> tokio::task::JoinHandle<u32> {

        let port = self.port;
        let node_id = self.node_id.clone();

        let peer_connected = self.peer_connected.clone();
        let peer_server_addr = self.peer_server_addr.clone();
        let write_streams = self.write_streams.clone();
        let msg_incoming_tx = self.msg_incoming_tx.clone();
        let msg_hashes = self.msg_hashes.clone();
        let server_addr = self.server_addr.clone();

        tokio::spawn(async move {

            let node_id = node_id.clone();
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}",port)).await.expect("Failed to bind the socket to the address");

            println!("Server listening on port {}",port);

            loop {

                let peer_connected = peer_connected.clone();
                let peer_server_addr = peer_server_addr.clone();
                let write_streams = write_streams.clone();
                let msg_incoming_tx = msg_incoming_tx.clone();
                let msg_hashes = msg_hashes.clone();
                let server_addr = server_addr.clone();

                if let Ok((stream, addr)) = listener.accept().await {
                    println!("New Peer Connected: {addr}");
                    tokio::spawn(async move {
                        Self::handle_connection(stream, node_id, peer_connected, peer_server_addr, write_streams, msg_incoming_tx,msg_hashes,server_addr).await;
                    });
                }
            }
        })
    }

    async fn handle_connection (
        mut stream: tokio::net::TcpStream, 
        node_id: Uuid, 
        peer_connected: Arc<tokio::sync::Mutex<Vec<Uuid>>>,
        peer_server_addr: Arc<tokio::sync::Mutex<Vec<String>>>,
        write_streams: Arc<tokio::sync::Mutex<HashMap<Uuid, OwnedWriteHalf>>>,
        msg_incoming_tx: crossbeam_channel::Sender<Message>,
        msg_hashes: Arc<tokio::sync::Mutex<HashMap<String,bool>>>,
        server_addr: String,
    ) {

        tokio::spawn(async move {
            let protocol = Self::detect_protocol(&mut stream).await;

            match protocol {
                Protocol::Handshake => {
                    Self::handle_handshake(stream,  peer_connected, peer_server_addr, node_id, write_streams, msg_incoming_tx, msg_hashes, server_addr).await;
                },
                _ => {
                    println!("Unknown protocol or format")
                }
            }
        });

    }

    async fn handle_handshake(
        mut stream: tokio::net::TcpStream,
        peer_connected: Arc<tokio::sync::Mutex<Vec<Uuid>>>,
        peer_server_addr: Arc<tokio::sync::Mutex<Vec<String>>>,
        node_id: Uuid,
        write_streams: Arc<tokio::sync::Mutex<HashMap<Uuid, OwnedWriteHalf>>>,
        msg_incoming_tx: crossbeam_channel::Sender<Message>,
        msg_hashes: Arc<tokio::sync::Mutex<HashMap<String,bool>>>,
        server_addr: String
    ) {

        let handshake_msg = format!("HNDSHK 10 {} {}", node_id, server_addr);
        let msg_len = handshake_msg.as_bytes().len();
        let mut peer_request = vec![0 as u8; msg_len];

        if let Err(e) = stream.read_exact(&mut peer_request).await {
            println!("Handshake failed due to {e}");
            return
        }
        let response = String::from_utf8(peer_request).unwrap();
        let response: Vec<&str> = response.split(" ").collect();
        match response[..] {
            // if proper request
            ["HNDSHK", "01", uuid, peer_addr] => {
                let mut peer_connected = peer_connected.lock().await;
                let peer_uuid = Uuid::from_str(uuid).unwrap();
                if !peer_connected.contains(&peer_uuid) {

                    peer_connected.push(peer_uuid);

                    println!("Peer {uuid} accepted with server ip {peer_addr}");

                    // respond with success message
                    stream.write_all(handshake_msg.as_bytes()).await.unwrap();

                    let (read_half, write_half) = stream.into_split();
                    let mut write_streams = write_streams.lock().await;

                    write_streams.insert(peer_uuid, write_half); // add to write_streams for writing to the clients

                    let mut peer_server_addr = peer_server_addr.lock().await;
                    peer_server_addr.push(peer_addr.to_string());

                    Self::peer_receiver(read_half, msg_incoming_tx, msg_hashes).await;
                }
            }
            _ => {
                println!("Invalid response. Handshake rejected");
                let handshake_msg = format!("HNDSHK 00 {} {}", node_id, server_addr);
                stream.write_all(handshake_msg.as_bytes()).await.unwrap();
            }
        }
    }

    async fn detect_protocol(stream: &mut tokio::net::TcpStream) -> Protocol {
        let mut buf = [0; 1024];
        if let Ok(n) = stream.peek(&mut buf).await {
            if n >= 7 && &buf[..7] == b"HNDSHK " {
                Protocol::Handshake
            } else {
                Protocol::Unknown
            }
        } else {
            Protocol::Unknown
        }
    }

    // Connecting to peer server. if successful use this thread to recieve messages and add the WriteHalf to collection for writing
    pub async fn add_peer(&self, addr: String) {

        let node_id = self.node_id;


        let peer_connected = self.peer_connected.clone();
        let write_streams = self.write_streams.clone();
        let msg_incoming_tx = self.msg_incoming_tx.clone();
        let msg_hashes = self.msg_hashes.clone();
        let server_addr = self.server_addr.clone();
        let peer_server_addr = self.peer_server_addr.clone();

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
            let handshake_msg = format!("HNDSHK 01 {} {}",node_id, server_addr);
            let msg_len = handshake_msg.as_bytes().len();
            stream.write_all(handshake_msg.as_bytes()).await.unwrap();


            let mut peer_response = vec![0 as u8; msg_len];

            if let Err(e) = stream.read_exact(&mut peer_response).await {
                println!("Handshake failed due to {e}");
                return
            }

            let response = String::from_utf8(peer_response).unwrap();
            let tokens: Vec<&str> = response.split(" ").into_iter().filter(|x| !x.is_empty()).collect();

            match tokens[..] {
                ["HNDSHK", "10", uuid, peer_addr] => {
                    let mut peer_connected = peer_connected.lock().await;
                    let peer_uuid = Uuid::from_str(uuid).unwrap();
                    if !peer_connected.contains(&peer_uuid) {

                        peer_connected.push(peer_uuid);

                        println!("Peer {peer_uuid} accepted with server ip {peer_addr}");

                        let (read_half, write_half) = stream.into_split();
                        let mut write_streams = write_streams.lock().await;

                        write_streams.insert(peer_uuid, write_half); // add to write_streams for writing to the clients

                        let mut peer_server_addr = peer_server_addr.lock().await;
                        peer_server_addr.push(peer_addr.to_string());

                        Self::peer_receiver(read_half, msg_incoming_tx, msg_hashes).await;
                        return
                    }
                },
                ["HNDSHK", "00", uuid, peer_addr] => {
                    println!("Handshake rejected from the peer {uuid} with peer server address {peer_addr}. may be already connected")
                },
                _ => {
                    println!("Invalid response or Handshake already established")
                },
            }
        });
    }

    async fn peer_receiver(mut stream: OwnedReadHalf, 
        msg_incoming_tx: crossbeam_channel::Sender<Message>,
        msg_hashes: Arc<tokio::sync::Mutex<HashMap<String,bool>>>
    ) {
        let msg_hashes = msg_hashes.clone();
        tokio::spawn(async move {
            loop {

                let mut buff = vec![0 as u8; 8];
                if let Err(e) = stream.read_exact(&mut buff).await {
                    eprintln!("Connection closed. Failed to recieve the message: {e}");
                    break;
                }
                let mut msg_len = [0u8; std::mem::size_of::<usize>()];
                msg_len.copy_from_slice(&buff);
                let msg_len = usize::from_be_bytes(msg_len);

                buff.clear();
                buff.resize(msg_len, 0);

                if let Err(e) = stream.read_exact(&mut buff).await {
                    eprintln!("Connection closed. Failed to recieve the message: {e}");
                    break;
                }

                let msg: Message= serde_json::from_str(str::from_utf8(&buff).unwrap()).unwrap();
                // continue if the message is already recieved

                let data_hash = msg.message_hash.clone();

                let mut msg_hashes = msg_hashes.lock().await;
                if let Some(_) = msg_hashes.get(&data_hash) {
                    continue;
                }

                msg_hashes.insert(data_hash, true);

                if let Err(e) = msg_incoming_tx.send(msg) {
                    println!("Failed to send the message from client {} to message reciever due to {e}", stream.peer_addr().unwrap())
                }
            }
        });
    }

    // pub fn take_reciever(&mut self) ->crossbeam_channel::Receiver<Message> {
    //     self.msg_incoming_rx.clone()
    // }

    // pub fn take_sender(&mut self) -> crossbeam_channel::Sender<Message> {
    //     self.msg_outgoing_tx.clone()
    // }

    async fn send_msg(msg_outgoing_rx: crossbeam_channel::Receiver<Message>, 
        write_streams: Arc<tokio::sync::Mutex<HashMap<Uuid, OwnedWriteHalf>>>,
        msg_hashes: Arc<tokio::sync::Mutex<HashMap<String,bool>>>,
    ) {
        let write_streams = write_streams.clone();
        let msg_hashes = msg_hashes.clone();

        tokio::spawn(async move {
            while let Ok(data) = msg_outgoing_rx.recv() {

                let mut msg_hashes = msg_hashes.lock().await;
                let data_hash = data.message_hash.clone();

                // Add data hash to already seen
                msg_hashes.insert(data_hash, true);

                let msg = Self::encode_msg_bytes(&data);
                let mut write_stream = write_streams.lock().await;

                let mut clients_to_remove = vec![];

                for (&uuid, stream) in write_stream.iter_mut() {
                    if let Err(e) = stream.write_all(&msg).await {
                        println!("Failed to write to the peer {}: {e}",stream.peer_addr().unwrap());
                        clients_to_remove.push(uuid)
                    }
                }

                for &uuid in clients_to_remove.iter().rev() {
                    write_stream.remove(&uuid);
                } 
            }
        });
    }

    fn encode_msg_bytes(data: &Message) -> Vec<u8> {
        let msg = serde_json::to_string(&data).unwrap();
        let header = (msg.len()).to_be_bytes();
        let mut msg_vec = Vec::with_capacity(msg.len() + header.len());
        msg_vec.extend_from_slice(&header);
        msg_vec.extend_from_slice(msg.as_bytes());
        msg_vec
    }
}