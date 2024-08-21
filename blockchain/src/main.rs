use std::{io::{self, Write}, process};
mod blockchain;


fn main() {
    let mut miner_addr = String::new();
    let mut difficulty = String::new();
    let mut choice = String::new();

    println!("Generating a genesis block...");

    print!("Input miner address: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut miner_addr).expect("Failed to read the miner address");

    print!("Difficulty: ");
    io::stdout().flush().expect("Failed to flush the stdout.");
    io::stdin().read_line(&mut difficulty).expect("Failed to read the difficulty");

    let diff = difficulty
        .trim()
        .parse::<u32>()
        .expect("we need an integer");

    let mut chain = blockchain::Chain::new(miner_addr.trim().to_string(), diff);

    loop {
        println!("MENU");
        println!("1) New Transaction");
        println!("2) Mine block");
        println!("3) Change Difficulty");
        println!("4) Change Reward");
        println!("5) Reveal the chain");
        println!("0) Exit");
        print!("Enter your choice: ");
        io::stdout().flush().expect("Failed to flush the stdout");
        choice.clear();
        io::stdin().read_line(&mut choice).expect("Failed to read the choice");
        println!("");

        match choice.trim().parse().unwrap_or(-1) {
            0 => {
                println!("exiting!");
                process::exit(0);
            }
            1 => {
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
            2 => {
                println!("Generating block...");
                let res = chain.generate_new_block();
                match res {
                    true => println!("Block added successfully"),
                    false => println!("Block failed to add")
                }
            }
            3 => {
                let mut new_diff = String::new();
                print!("Enter new difficulty: ");
                io::stdout().flush().expect("Failed to flush the stdout.");
                io::stdin().read_line(&mut new_diff).expect("Failed to read the new difficulity");
                let res = chain.update_difficulty(new_diff.trim().parse().unwrap());
                match res {
                    true => println!("Updated Difficulty"),
                    false => println!("Failed to update the difficulty")
                }
            }
            4 => {
                let mut new_reward = String::new();
                print!("Enter new reward: ");
                io::stdout().flush().expect("Failed to flush the stdout.");
                io::stdin().read_line(&mut new_reward).expect("Failed to read the reward");
                let res = chain.update_reward(new_reward.trim().parse().unwrap());
                match res {
                    true => println!("Updated reward"),
                    false => println!("Failed to update the reward")
                }
            }
            5 => {
                chain.reveal_chain();
            }
            _ => println!("Invalid option please retry")
        }
    }
}