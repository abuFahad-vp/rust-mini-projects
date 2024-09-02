use std::sync::{Arc, Mutex};
use rocksdb::{Options, DB};
use local_ip_address::local_ip;
use crate::blockchain::blockchain_core::Chain;
use crate::template::{self, MenuBuilder};
use crate::utils::get_value;
use crate::Args;

pub fn blockchain_app(args: Args) {

    let db_path = "amanah.db";
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    let db = DB::open(&db_opts, db_path).unwrap();

    let chain = Arc::new(Mutex::new(
        Chain::new(db)
    ));

    let peers = Arc::new(Mutex::new(Vec::new()));
    get_peers(peers.clone());
    for peer in peers.lock().unwrap().iter() {
        println!("peers = {}",peer);
    }

    blockchain_start(chain, args).run_menu();
}

fn blockchain_start(chain: Arc<Mutex<Chain>>, args: Args) -> MenuBuilder {
    let header = format!("
    Welcome to shuranetwork!!
    IP ADDRESS: {}:{}
    ", local_ip().expect("Failed to get the local ip. Internal Error"), args.port);

    let mut blockchain_page = template::MenuBuilder::new();
    blockchain_page.set_header(header);

    blockchain_page.add("0", "Exit", || {
        false
    });

    let chain_clone = chain.clone();
    blockchain_page.add("1", "New Transaction",
        move || {
            let chain = chain_clone.clone();
            new_transaction(chain);
            true
    });

    let chain_clone = chain.clone();
    blockchain_page.add("2", "Mine block", {
        move || {
            let chain = chain_clone.clone();
            mine_block(chain);
            true
    }});

    let chain_clone = chain.clone();
    blockchain_page.add("3", "Change difficulty", {
        move || {
            let chain = chain_clone.clone();
            change_difficulty(chain);
            true
    }});

    let chain_clone = chain.clone();
    blockchain_page.add("4", "Change Reward", {
        move || {
            let chain = chain_clone.clone();
            change_reward(chain);
            true
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("5", "reveal chain", {
        move || {
            let chain = chain_clone.lock().unwrap();
            chain.reveal_chain();
            true
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("6", "Show height", {
        move || {
            let chain = chain_clone.lock().unwrap();
            println!("height: {}",chain.get_height());
            true
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("7", "Show hash by index", {
        move || {
            let chain = chain_clone.clone();
            show_hash_by_index(chain);
            true
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("8", "Change miner address", {
        move || {
            let chain = chain_clone.clone();
            change_miner_address(chain);
            true
        }
    });
    blockchain_page
}

fn get_peers(peers: Arc<Mutex<Vec<String>>>) {

    // adding peer
    let peers_clone = peers.clone();
    let mut peer_page = template::MenuBuilder::new();

    peer_page.add("1", "Add peer", move || {
        let mut peers = peers_clone.lock().unwrap();
        peers.push(get_value("Add peer address: "));
        true
    });

    peer_page.add("0", "Continue" , || {
        false
    });
    peer_page.run_menu();
}

fn show_hash_by_index(chain: Arc<Mutex<Chain>>) {

    let choice = get_value("Input index: ");

    let chain = chain.lock().unwrap();
    if let Ok(choice) = choice.trim().parse::<u32>() {
        println!("hash of index {}: {:?}",choice, chain.get_hash_by_index(choice));
    } else {
        println!("Invalid input");
    }
}

fn new_transaction(chain: Arc<Mutex<Chain>>) {
    let sender = get_value("Enter Sender Address: ");
    let reciever = get_value("Enter Reciever Address: ");
    let amount = get_value("Enter the amount: ");

    let mut chain = chain.lock().unwrap();

    let res = chain.new_transaction(
        sender.trim().to_string(),
        reciever.trim().to_string(),
        amount.trim().parse().unwrap()
    );

    match res {
        true => println!("Transaction added"),
        false => println!("Transaction failed"),
    }
}

fn mine_block(chain: Arc<Mutex<Chain>>) {
    println!("Generating block...");
    let mut chain = chain.lock().unwrap();
    let res = chain.generate_new_block();
    match res {
        true => println!("Block added successfully"),
        false => println!("Block failed to add")
    }
}

fn change_difficulty(chain: Arc<Mutex<Chain>>) {

    let new_diff = get_value("Enter new difficulty: ");
    
    let mut chain = chain.lock().unwrap();
    let res = chain.update_difficulty(new_diff.trim().parse().unwrap());
    match res {
        true => println!("Updated Difficulty"),
        false => println!("Failed to update the difficulty")
    }
}

fn change_reward(chain: Arc<Mutex<Chain>>) {
    let new_reward = get_value("Enter new reward: ");

    let mut chain = chain.lock().unwrap();
    let res = chain.update_reward(new_reward.trim().parse().unwrap());
    match res {
        true => println!("Updated reward"),
        false => println!("Failed to update the reward")
    }
}
fn change_miner_address(chain: Arc<Mutex<Chain>>) {
    let miner_addr = get_value("Enter new miner address: ");
    let mut chain = chain.lock().unwrap();
    chain.update_miner_address(miner_addr);
}