use std::future::Future;
use std::io::Write;
use std::collections::HashMap;
use std::pin::Pin;

pub struct MenuBuilder {
    header: String,
    menus: std::collections::HashMap<String, (&'static str, Box<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send>)>,
}

impl MenuBuilder {
    pub fn new() -> Self {
        MenuBuilder{menus: HashMap::new(), header: "".to_string()}
    }

    // calling set_header function override the key :show if exists
    pub fn set_header(&mut self, header: String) {
        self.header = header;
    }

    pub fn add<F, Fut>(&mut self, key: &'static str, desc: &'static str, fx: F)
    where 
        F: Fn() -> Fut + 'static + Send,
        Fut: Future<Output = bool> + 'static + Send
    {
        let _ = self.menus.insert(key.to_string(), (desc, Box::new(move || Box::pin(fx()))));
    }

    pub async fn run_menu(&self) {
        println!("{}", self.header);
        loop {
            println!("");
            let mut keys: Vec<u32> = self.menus.keys().map(|x| {
                x.parse::<u32>().unwrap()
            }).collect();
            keys.sort();
            for key in keys {
                let key = key.to_string();
                println!("{key}: {}",self.menus[&key].0)
            }
            println!("");

            let mut choice = String::new();
            std::io::stdout().flush().expect("Failed to flush the output: INTERNAL ERROR");
            print!("input: "); // clear the screen
            std::io::stdout().flush().expect("Failed to flush the stdout: INTERNAL ERROR");
            std::io::stdin().read_line(&mut choice).expect("Failed to read from stdin: INTERNAL ERROR");
            match self.menus.get(choice.trim()) {
                Some((_,fx)) => {
                    if !(fx().await) {
                        break;
                    }
                }
                None => println!("Invalid option")
            }
            // print!("\x1B[2J\x1B[1;1H"); // clear the screen
            // std::io::stdout().flush().expect("Failed to flush the stdout: INTERNAL ERROR");
        }
    }
}