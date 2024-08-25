use std::io::Write;
use std::collections::HashMap;

pub struct MenuBuilder {
    header: String,
    menus: HashMap<i32, (&'static str, Box<dyn Fn() -> bool>)>
}

impl MenuBuilder {
    pub fn new() -> Self {
        MenuBuilder{menus: HashMap::new(), header: "".to_string()}
    }

    pub fn set_header(&mut self, header: String) {
        self.header = header;
    }

    pub fn add<F>(&mut self, key: i32, desc: &'static str, fx: F)
    where 
        F: Fn() -> bool + 'static
    {
        let _ = self.menus.insert(key, (desc, Box::new(fx)));
    }

    pub fn run_menu(&self) {
        println!("{}", self.header);
        loop {
            println!("");
            for (key, (desc,_)) in &self.menus {
                println!("{key}: {}",desc)
            }
            println!("");

            let mut choice_raw = String::new();
            std::io::stdout().flush().expect("Failed to flush the output");
            std::io::stdin().read_line(&mut choice_raw).expect("Failed to read from stdin: INTERNAL ERROR");
            if let Ok(choice) = choice_raw.trim().parse::<i32>() {
                match self.menus.get(&choice) {
                    Some((_,fx)) => {
                        if !fx(){
                            break;
                        }
                    }
                    None => println!("Invalid option")
                }
            } else {
                println!("Invalid option")
            }
        }
    }
}