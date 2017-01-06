use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::{thread, time};

use pocsag::{TimeSlots, Message, Generator};
use transmitter::Transmitter;
use config::Config;

enum Command {
    Enqueue(Message),
    SetTimeSlots(TimeSlots),
    Stop
}

#[derive(Clone)]
pub struct Scheduler {
    tx: Sender<Command>,
    scheduler: Arc<Mutex<SchedulerCore>>,
}

struct SchedulerCore {
    rx: Receiver<Command>,
    queue: VecDeque<Message>,
    slots: TimeSlots
}

impl Scheduler {
    pub fn new(_: &Config) -> Scheduler {
        let (tx, rx) = channel();

        let core = SchedulerCore {
            rx: rx,
            queue: VecDeque::new(),
            slots: TimeSlots::new()
        };

        Scheduler {
            tx: tx,
            scheduler: Arc::new(Mutex::new(core))
        }
    }

    pub fn set_time_slots(&self, slots: TimeSlots) {
        info!("{:?}", slots);
        self.tx.send(Command::SetTimeSlots(slots)).unwrap();
    }

    pub fn enqueue(&self, msg: Message) {
        info!("{:?}", msg);
        self.tx.send(Command::Enqueue(msg)).unwrap();
    }

    pub fn stop(&self) {
        self.tx.send(Command::Stop).unwrap();
    }

    pub fn run<T: Transmitter>(&self, transmitter: T) {
        self.scheduler.lock().unwrap().run(transmitter);
    }
}

impl SchedulerCore {
    pub fn run<T: Transmitter>(&mut self, mut transmitter: T) {
        info!("Scheduler started.");
        loop {
            loop {
                match self.rx.try_recv() {
                    Ok(Command::Enqueue(msg)) => { self.queue.push_back(msg); },
                    Ok(Command::SetTimeSlots(slots)) => { self.slots = slots; },
                    Ok(Command::Stop) => { return; },
                    Err(TryRecvError::Empty) => { break; },
                    Err(TryRecvError::Disconnected) => { return; }
                }
            }

            while let Some(message) = self.queue.pop_front() {
                let generator = Generator::new(vec![message]);
                info!("Transmitting...");
                transmitter.send(generator);
                info!("Transmission completed.")
            }

            thread::sleep(time::Duration::from_millis(1000));
        }
    }
}
