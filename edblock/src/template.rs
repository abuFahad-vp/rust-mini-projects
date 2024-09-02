use std::future::Future;
use std::io::Write;
use std::collections::HashMap;
use std::pin::Pin;

pub struct MenuBuilder {
    header: String,
    menus: std::collections::HashMap<&'static str, (&'static str, Box<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send>)>,
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
        let _ = self.menus.insert(key, (desc, Box::new(move || Box::pin(fx()))));
    }

    pub async fn run_menu(&self) {
        println!("{}", self.header);
        loop {
            println!("");
            for (key, (desc,_)) in &self.menus {
                println!("{key}: {}",desc)
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