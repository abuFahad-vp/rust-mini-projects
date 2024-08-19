use std::{io::{self, ErrorKind, Read, Write}, net::TcpStream, sync::mpsc::{self, TryRecvError}, thread};

const MSG_SIZE: usize = 1024;

fn main() {
    let mut client = TcpStream::connect("127.0.0.1:8080").expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0;MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("failed to convert to utf-8");
                println!("{}", msg);
                println!("Write ur message: ");
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with the server is ended.");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to the socket failed");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
    });

    loop {
        println!("Write ur Message: ");
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {break}
    }
    println!("Bye Bye!");
}