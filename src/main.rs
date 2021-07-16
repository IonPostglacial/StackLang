mod stackmachine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        let mut machine = stackmachine::new();

        match machine.eval(&args[1]) {
            Ok(()) => machine.print_stack(),
            Err(err) => println!("error: {:?}", err),
        }
    }
}
