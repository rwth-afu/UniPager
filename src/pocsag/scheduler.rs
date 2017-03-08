use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::mpsc::{TryRecvError, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;

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
    queue: VecDeque<Message>,
    stop: bool
}

impl Scheduler {
    pub fn new(_: &Config) -> Scheduler {
        let (tx, rx) = channel();

        let core = SchedulerCore {
            rx: rx,
            slots: TimeSlots::new(),
            queue: VecDeque::new(),
            stop: false
        };

        Scheduler {
            tx: tx,
            scheduler: Arc::new(Mutex::new(core))
        }
    }

    pub fn start(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
        use transmitter::*;
        use config::Transmitter;

        thread::spawn(move || {
            match config.transmitter {
                Transmitter::Dummy =>
                    scheduler.run(DummyTransmitter::new(&config)),
                Transmitter::AudioGpio =>
                    scheduler.run(AudioGpioTransmitter::new(&config)),
                Transmitter::AudioRs232 =>
                    scheduler.run(AudioRs232Transmitter::new(&config)),
                Transmitter::Raspager =>
                    scheduler.run(RaspagerTransmitter::new(&config)),
                Transmitter::C9000 =>
                    scheduler.run(C9000Transmitter::new(&config)),
                Transmitter::STM32Pager =>
                    scheduler.run(STM32Transmitter::new(&config))
            };
        })
    }

    pub fn set_time_slots(&self, slots: TimeSlots) -> bool {
        info!("Set {:?}", slots);
        self.tx.send(Command::SetTimeSlots(slots)).is_ok()
    }

    pub fn message(&self, msg: Message) -> bool {
        info!("Received {:?}", msg);
        self.tx.send(Command::Message(msg)).is_ok()
    }

    pub fn stop(&self) -> bool {
        self.tx.send(Command::Stop).is_ok()
    }

    pub fn run<T: Transmitter>(&self, transmitter: T) {
        self.scheduler.lock().unwrap().run(transmitter);
    }
}

impl SchedulerCore {
    pub fn run<T: Transmitter>(&mut self, mut transmitter: T) {
        info!("Scheduler started.");
        while !self.stop {
            let mut message = self.queue.pop_front();

            while message.is_none() {
                match self.rx.recv() {
                    Ok(Command::Message(msg)) => {
                        message = Some(msg);
                    },
                    Ok(Command::SetTimeSlots(slots)) => {
                        self.slots = slots;
                    },
                    Ok(Command::Stop) => { return; },
                    Err(_) => { return; }
                }
            }

            if self.slots.is_current_allowed() { /* transmit immediately */ }
            else if let Some(next_slot) = self.slots.next_allowed() {
                let mut duration = next_slot.duration_until();

                info!("Waiting {} seconds until {:?}...",
                      duration.as_secs(), next_slot);

                // Process other commands while waiting for the time slot
                'waiting: while !next_slot.active() {
                    duration = next_slot.duration_until();

                    match self.rx.recv_timeout(duration) {
                        Ok(Command::Message(msg)) => {
                            self.queue.push_back(msg);
                        },
                        Ok(Command::SetTimeSlots(slots)) => {
                            self.slots = slots;
                        },
                        Ok(Command::Stop) => { return; },
                        Err(RecvTimeoutError::Disconnected) => { return; }
                        Err(RecvTimeoutError::Timeout) => { break 'waiting; }
                    }
                }
            }
            else {
                warn!("No allowed time slots! Sending anyway...");
            }

            status!(transmitting: true);
            transmitter.send(Generator::new(self, message.unwrap()));
            status!(transmitting: false);
        }
    }
}

impl MessageProvider for SchedulerCore {
    fn next(&mut self) -> Option<Message> {
        if !self.slots.is_current_allowed() {
            return None;
        }

        loop {
            match (*self).rx.try_recv() {
                Ok(Command::Message(msg)) =>{
                    self.queue.push_back(msg);
                }
                Ok(Command::SetTimeSlots(slots)) => {
                    self.slots = slots;
                },
                Ok(Command::Stop) => {
                    self.stop = true;
                    return None;
                },
                Err(TryRecvError::Disconnected) => {
                    self.stop = true;
                    return None;
                },
                Err(TryRecvError::Empty) => { break; }
            };
        }

        self.queue.pop_front()
    }
}
