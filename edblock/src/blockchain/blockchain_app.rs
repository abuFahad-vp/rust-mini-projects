use std::io;
use std::io::Write;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use rocksdb::{Options, DB};

use crate::blockchain::blockchain_core::Chain;
use crate::template;

pub fn blockchain_app() {
    let mut miner_addr = String::new();
    let mut difficulty = String::new();

    print!("Input miner address: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut miner_addr).expect("Failed to read the miner address");

    print!("Difficulty: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut difficulty).expect("Failed to read the difficulty");

    // adding peer
    let peers = Arc::new(Mutex::new(Vec::<String>::new()));
    let peers_clone = peers.clone();
    let mut pre_page = template::MenuBuilder::new();

    pre_page.add("1", "Add peer", move || {
        let mut peers = peers_clone.lock().unwrap();
        let mut peer = String::new();
        print!("peer address: ");
        io::stdout().flush().expect("Failed to flush the stdout.");
        io::stdin().read_line(&mut peer).expect("Failed to read the difficulty");
        peers.push(peer);
        true
    });

    pre_page.add("0", "Exit" , || {
        false
    });

    pre_page.run_menu();

    for (i, peer) in peers.lock().unwrap().iter().enumerate() {
        println!("peer {i}:{peer}");
    }

    let diff = difficulty
        .trim()
        .parse::<u32>()
        .expect("we need an integer");
    
    let db_path = "amanah.db";
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    let db = DB::open(&db_opts, db_path).unwrap();
    let chain = Rc::new(RefCell::new(
        Chain::new(miner_addr.trim().to_string(), diff, db)
    ));

    let mut blockchain_page = template::MenuBuilder::new();
    blockchain_page.set_header("MENU".to_string());

    blockchain_page.add("0", "Exit", || {
        false
    });

    blockchain_page.add("1", "New Transaction", {
        let chain = Rc::clone(&chain);
        move || {
            new_transaction(chain.clone());
            true
    }});

    blockchain_page.add("2", "Mine block", {
        let chain = Rc::clone(&chain);
        move || {
            mine_block(chain.clone());
            true
    }});

    blockchain_page.add("3", "Change difficulty", {
        let chain = Rc::clone(&chain);
        move || {
            change_difficulty(chain.clone());
            true
    }});

    blockchain_page.add("4", "Change Reward", {
        let chain = Rc::clone(&chain);
        move || {
            change_reward(chain.clone());
            true
        }
    });

    blockchain_page.add("5", "reveal chain", {
        let chain = Rc::clone(&chain);
        move || {
            chain.borrow_mut().reveal_chain();
            true
        }
    });

    blockchain_page.add("6", "Show height", {
        let chain = Rc::clone(&chain);
        move || {
            println!("height: {}",chain.borrow().get_height());
            true
        }
    });

    blockchain_page.add("7", "Show hash by index", {
        let chain = Rc::clone(&chain);
        move || {
            show_hash_by_index(chain.clone());
            true
        }
    });

    blockchain_page.run_menu();
}

fn show_hash_by_index(chain: Rc<RefCell<Chain>>) {
    print!("input index: ");
    std::io::stdout().flush().expect("Failed to flush the stdout");
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice).expect("Failed to read the index");
    if let Ok(choice) = choice.trim().parse::<u32>() {
        println!("hash of index {}: {:?}",choice, chain.borrow().get_hash_by_index(choice));
    } else {
        println!("Invalid input");
    }
}

fn new_transaction(chain: Rc<RefCell<Chain>>) {
    let mut sender = String::new();
    let mut reciever = String::new();
    let mut amount  = String::new();

    print!("enter sender address: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut sender).expect("Failed to read the sender address");
    print!("enter reciever address: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut reciever).expect("Failed to read the reciever address");
    print!("Enter amount: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut amount).expect("Failed to read the amount");

    let res = chain.borrow_mut().new_transaction(
        sender.trim().to_string(),
        reciever.trim().to_string(),
        amount.trim().parse().unwrap()
    );

    match res {
        true => println!("Transaction added"),
        false => println!("Transaction failed"),
    }
}

fn mine_block(chain: Rc<RefCell<Chain>>) {
    println!("Generating block...");
    let res = chain.borrow_mut().generate_new_block();
    match res {
        true => println!("Block added successfully"),
        false => println!("Block failed to add")
    }
}

fn change_difficulty(chain: Rc<RefCell<Chain>>) {
    let mut new_diff = String::new();
    print!("Enter new difficulty: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut new_diff).expect("Failed to read the new difficulity");
    let res = chain.borrow_mut().update_difficulty(new_diff.trim().parse().unwrap());
    match res {
        true => println!("Updated Difficulty"),
        false => println!("Failed to update the difficulty")
    }
}

fn change_reward(chain: Rc<RefCell<Chain>>) {
    let mut new_reward = String::new();
    print!("Enter new reward: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut new_reward).expect("Failed to read the reward");
    let res = chain.borrow_mut().update_reward(new_reward.trim().parse().unwrap());
    match res {
        true => println!("Updated reward"),
        false => println!("Failed to update the reward")
    }
}