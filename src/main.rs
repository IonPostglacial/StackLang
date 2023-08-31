mod stack;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 {
        let mut machine = stack::Machine::new();

        match &*args[1] {
            "-i" => match machine.eval(&args[2]) {
                Ok(stack) => println!("{:?}", stack),
                Err(err) => println!("error: {:?}", err),
            },
            "-f" => {
                let file_content = std::fs::read_to_string(&args[2]);
                match file_content {
                    Ok(text) => {
                        match machine.eval(&text) {
                            Ok(stack) => println!("{:?}", stack),
                            Err(err) => println!("error: {:?}", err),
                        }
                    }
                    Err(err) => {
                        println!("error: {:?}", err)
                    }
                }
            },
            _ => {}
        }
        
    }
}
