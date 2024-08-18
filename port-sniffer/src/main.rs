use core::time;
use std::{env, net::{IpAddr, SocketAddr, TcpStream}, process, str::FromStr, thread};

/*
exe -h
exe -t 100 192.168.1.1
exe 192.168.1.1
*/

struct Arguments {
    ip_addr: Option<IpAddr>,
    threads: usize,
    flag: &'static str
}
const MAX_PORT_NUMBER: u16 = 65535;

impl Arguments {
    fn new(args: &[String]) -> Arguments {
        if args.len() < 2 || args.len() > 4 {
            return Arguments{ip_addr: None, threads: 0, flag: "-h"} 
        }

        if let Ok(ip_addr) = IpAddr::from_str(&args[1]) {
            return Arguments {ip_addr: Some(ip_addr), threads: 4, flag: ""};
        }
        if &args[1] == "-t" {
            if let Ok(threads) = &args[2].parse::<usize>() {
                if let Ok(ip_addr) = IpAddr::from_str(&args[3]) {
                    return Arguments {ip_addr: Some(ip_addr), threads: *threads, flag: "-j"};
                }
            }
        }
        return Arguments{ip_addr: None, threads: 0, flag: "-h"} 
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let arguments = Arguments::new(&args);
    if arguments.flag == "-h" {
        println!("Usage: \"port-sniffer.exe -t 100 192.168.1.1\" where -t to select how many threads you want
        \n\r    or \"port-sniffer.exe 192.168.1.1\" to use default threads
        \n ");
        process::exit(0)
    }
    
    let Arguments {ip_addr, threads,flag:_ } = arguments;
    let ip_addr = ip_addr.unwrap_or_else(|| {
        println!("Have some internal issue. Using default ip");
        IpAddr::from_str("0.0.0.0").expect("Failed: Internal Error")
    });
    let num_threads = if threads > 0 {threads} else {
        println!("Invalid thread count. using default number of threads: 4"); 4
    } as u16;

    let mut threads = vec![];

    for start_point in 0..num_threads{
        threads.push(thread::spawn(move || {scan(ip_addr, start_point, num_threads);}));
    }

    for t in threads {
        t.join().unwrap();
    }

}

fn scan(ip_addr: IpAddr, start_point: u16, num_threads: u16) {
    let mut port = start_point + 1;
    let time_out = time::Duration::from_nanos(1);
    loop {
        match TcpStream::connect_timeout(&SocketAddr::new(ip_addr, port), time_out) {
            Ok(_) => {
                println!("{} is open", port);
            }
            Err(_) => {}
        }
        if (MAX_PORT_NUMBER - port) <= num_threads {
            break;
        }
        port += num_threads
    }
}