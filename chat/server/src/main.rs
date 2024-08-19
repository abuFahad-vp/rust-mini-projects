use std::{io::{ErrorKind, Read, Write}, net::TcpListener, sync::mpsc, thread};

const MSG_SIZE: usize = 1024;

fn main() {
    let server = TcpListener::bind("127.0.0.1:8080").expect("Listener failed to bind");
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    println!("Server started....");

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client connected: {:?}", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone the client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let mut msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let mut formatted_msg = format!("{} said: ", addr).into_bytes();
                        formatted_msg.append(&mut msg);
                        let msg = String::from_utf8(formatted_msg).expect("Invaild utf8 message");
                        println!("{}", msg);
                        tx.send(msg).expect("Failed to send msg to rx");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing the connection with {}",addr);
                        break;
                    }
                }
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }
    }
}
