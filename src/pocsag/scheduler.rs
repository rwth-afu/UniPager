use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use pocsag::{TimeSlots, Message, MessageProvider, Generator};
use transmitter::Transmitter;
use config::Config;

enum Command {
    Message(Message),
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
    slots: TimeSlots,
    stop: bool
}

impl Scheduler {
    pub fn new(_: &Config) -> Scheduler {
        let (tx, rx) = channel();

        let core = SchedulerCore {
            rx: rx,
            slots: TimeSlots::new(),
            stop: false
        };

        Scheduler {
            tx: tx,
            scheduler: Arc::new(Mutex::new(core))
        }
    }

    pub fn set_time_slots(&self, slots: TimeSlots) {
        info!("Set {:?}", slots);
        self.tx.send(Command::SetTimeSlots(slots)).unwrap();
    }

    pub fn message(&self, msg: Message) {
        info!("Received {:?}", msg);
        self.tx.send(Command::Message(msg)).unwrap();
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
            let mut message = None;

            while message.is_none() {
                match self.rx.recv() {
                    Ok(Command::Message(msg)) => { message = Some(msg); },
                    Ok(Command::SetTimeSlots(slots)) => { self.slots = slots; },
                    Ok(Command::Stop) => { return; },
                    Err(_) => { return; }
                }
            }

            if let Some(message) = message {
                let next_slot = self.slots.next_allowed();

                if let Some(next_slot) = next_slot {
                    info!("Waiting for {:?}...", next_slot);
                    thread::sleep(next_slot.duration_until());
                }
                else {
                    warn!("No allowed time slots! Sending anyway...");
                }

                info!("Transmitting...");
                let generator = Generator::new(self, message);
                transmitter.send(generator);
                info!("Transmission completed.");
            }

            if self.stop { return; }
        }
    }
}

impl MessageProvider for SchedulerCore {
    fn next(&mut self) -> Option<Message> {
        if !self.slots.is_current_allowed() {
            return None;
        }

        match (*self).rx.try_recv() {
            Ok(Command::Message(msg)) => Some(msg),
            Ok(Command::SetTimeSlots(slots)) => { self.slots = slots; self.next() },
            Ok(Command::Stop) => { self.stop = true; None },
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => { self.stop = true; None }
        }
    }
}
