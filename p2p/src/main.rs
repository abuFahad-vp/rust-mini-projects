use core::{str, time};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

const SIZE: usize = 50;

const PEERS: &[&str]  = &["127.0.0.1:8000", "127.0.0.1:8001", "127.0.0.1:8002"];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    thread::spawn(|| {
        client_reader();
    });
    server(args[1].clone());
}

fn client_reader() {
    for addr in PEERS {
        thread::spawn(move || {
            println!("Connecting to {addr}");
            let mut stream :Option<TcpStream> = None;
            while stream.is_none() {
                match TcpStream::connect(&addr) {
                    Ok(s) => {
                        println!("Successfully connected to the {addr:?}");
                        stream = Some(s)
                    }
                    Err(_) => {
                        println!("Retrying....");
                        thread::sleep(time::Duration::from_millis(200));
                    }
                };
            }
            let mut stream = stream.unwrap();
            loop {
                let mut buff = vec![0 as u8; SIZE];
                match stream.read_exact(&mut buff) {
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

fn server(port: String) {
    let listener = TcpListener::bind("127.0.0.1:".to_owned() + &port).unwrap();
    println!("Server is listening on port {}...",port);

    let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));
    let clients_clone = clients.clone();

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                println!("Client connected: {}",stream.peer_addr().unwrap());
                clients_clone.lock().unwrap().push(stream);
            }
        }
    });
    write_to_clients(clients);
}

fn write_to_clients(clients: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        let mut msg = String::new();

        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut msg).unwrap();

        let mut msg = msg.trim().as_bytes().to_vec();
        msg.resize(SIZE, 0);

        let clients = clients.lock().unwrap();

        for mut client in clients.iter() {
            if let Err(e) = client.write_all(&mut msg) {
                eprint!("Failed to send the message: {e}");
            }
        }
    }
}