use std::sync::mpsc;

enum Command {
    SayHello,
    Quit,
}

fn main() {
    println!("running simple channel");
    run_simple_channel();

    println!("running sending functions");
    run_sending_functions();
}

fn run_simple_channel() {
    let (tx, rx) = mpsc::channel::<Command>();

    let handle = std::thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            match command {
                Command::SayHello => println!("hello"),
                Command::Quit => {
                    println!("quiting");
                    break;
                }
            }
        }
    });

    for _ in 0..10 {
        tx.send(Command::SayHello).unwrap();
    }

    println!("sending quit");
    tx.send(Command::Quit).unwrap();

    handle.join().unwrap();
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum JobCommand {
    Quit,
    Run(Job),
}

fn hi_there() {
    println!("hi there");
}

fn run_sending_functions() {
    let (tx, rx) = mpsc::channel::<JobCommand>();
    let handle = std::thread::spawn(move || {
        while let Ok(commmand) = rx.recv() {
            match commmand {
                JobCommand::Run(job) => job(),
                JobCommand::Quit => break,
            }
        }
    });

    let job = || println!("Hello from the thread!");
    tx.send(JobCommand::Run(Box::new(job))).unwrap();
    tx.send(JobCommand::Quit).unwrap();
    handle.join().unwrap();
}
