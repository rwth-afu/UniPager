use rustyline::error::ReadlineError;
use rustyline::Editor;
use scheduler::Scheduler;
use message::{Message, MessageType, MessageSpeed, MessageFunc};

pub fn run(scheduler: Scheduler) -> bool {
    let mut rl = Editor::<()>::new();
    rl.load_history(".rp_history").ok();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                rl.save_history(".rp_history").unwrap();

                let msg = Message {
                    id: 0,
                    mtype: MessageType::AlphaNum,
                    speed: MessageSpeed::Baud(1200),
                    addr: 67544,
                    func: MessageFunc::AlphaNum,
                    text: line
                };

                scheduler.enqueue(msg);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C pressed. Good bye!");
                return false;
            },
            Err(ReadlineError::Eof) => {
                println!("Closing the prompt.");
                return true;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                return true;
            }
        }
    }
}
