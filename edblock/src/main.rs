mod template;

use clap::Parser;
use local_ip_address::local_ip;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8000, help = "port for running the node")]
    port: u32,
}

fn main() {
    let args = Args::parse();
    let mut start_page = template::MenuBuilder::new();
    start_page.set_header(starter_page_header(args.port));
    start_page.add(1, "Go Online", || {
        println!("GOING ONLINE....");
        true
    });

    start_page.add(0, "Exit", || {
        println!("Exiting...");
        false
    });
    start_page.run_menu();
}

fn starter_page_header(port: u32) -> String {
    format!("

    Welcome to shuranetwork!!
    IP ADDRESS: {}:{}
    NODE STATUS: OFFLINE

", local_ip().expect("Failed to get the local ip. Internal Error"), port)
}