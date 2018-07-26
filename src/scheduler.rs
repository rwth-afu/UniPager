use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::mpsc::{RecvTimeoutError, TryRecvError};
use std::thread::{self, JoinHandle};

use config::Config;
use message::{Message, MessageProvider};
use pocsag;
use transmitter::{self, Transmitter};
use timeslots::TimeSlots;
use queue::Queue;
use telemetry;

enum Command {
    Message(Message),
    SetTimeSlots(TimeSlots),
    Stop
}

#[derive(Clone)]
pub struct Scheduler {
    tx: Sender<Command>,
    scheduler: Arc<Mutex<SchedulerCore>>
}

struct SchedulerCore {
    rx: Receiver<Command>,
    slots: TimeSlots,
    queue: Queue,
    stop: bool,
    budget: usize
}

impl Scheduler {
    pub fn new(_: &Config) -> Scheduler {
        let (tx, rx) = channel();

        let core = SchedulerCore {
            rx: rx,
            slots: TimeSlots::new(),
            queue: Queue::new(),
            stop: false,
            budget: 0
        };

        Scheduler {
            tx: tx,
            scheduler: Arc::new(Mutex::new(core))
        }
    }

    pub fn start(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
        thread::spawn(move || {
            let transmitter = transmitter::from_config(&config);
            scheduler.scheduler.lock().unwrap().run(transmitter);
        })
    }

    pub fn test(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
        thread::spawn(move || {
            let transmitter = transmitter::from_config(&config);
            scheduler.scheduler.lock().unwrap().test(transmitter);
        })
    }

    pub fn set_time_slots(&self, slots: TimeSlots) -> bool {
        info!("Set {:?}", slots);
        telemetry_update!(config: |config: &mut telemetry::Config| {
            config.timeslots = slots.raw();
        });

        self.tx.send(Command::SetTimeSlots(slots)).is_ok()
    }

    pub fn message(&self, msg: Message) -> bool {
        info!("Received {:?}", msg);
        self.tx.send(Command::Message(msg)).is_ok()
    }

    pub fn stop(&self) -> bool {
        self.tx.send(Command::Stop).is_ok()
    }
}

impl SchedulerCore {
    pub fn run(&mut self, mut transmitter: Box<Transmitter>) {
        info!("Scheduler started.");
        while !self.stop {
            let mut message = self.queue.dequeue();

            while message.is_none() {
                match self.rx.recv() {
                    Ok(Command::Message(msg)) => {
                        message = Some(msg);
                    }
                    Ok(Command::SetTimeSlots(slots)) => {
                        self.slots = slots;
                    }
                    Ok(Command::Stop) |
                    Err(_) => {
                        return;
                    }
                }
            }

            // Calculate remaining time budget
            self.budget = self.slots.calculate_budget();

            if self.budget > 30 {
                /* transmit immediately */
            } else if let Some(next_slot) = self.slots.next_allowed() {
                let mut duration = next_slot.duration_until();

                debug!(
                    "Waiting {} seconds until {:?}...",
                    duration.as_secs(),
                    next_slot
                );

                // Process other commands while waiting for the time slot
                'waiting: while !next_slot.active() {
                    duration = next_slot.duration_until();

                    match self.rx.recv_timeout(duration) {
                        Ok(Command::Message(_msg)) => {
                            //self.queue.push_back(msg);
                            //status!(queue: self.queue.len() + 1);
                        }
                        Ok(Command::SetTimeSlots(slots)) => {
                            self.slots = slots;
                        }
                        Ok(Command::Stop) |
                        Err(RecvTimeoutError::Disconnected) => {
                            //self.queue.push_front(message.unwrap());
                            return;
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            break 'waiting;
                        }
                    }
                }

                self.budget = self.slots.calculate_budget();
            } else {
                warn!("No allowed time slots! Sending anyway...");
                self.budget = usize::max_value();
            }

            debug!("Available time budget: {}", self.budget);
            telemetry!(onair: true);
            transmitter.send(
                &mut *message.unwrap().generator(self)
            );
            telemetry!(onair: false);
        }
    }

    pub fn test(&mut self, mut transmitter: Box<Transmitter>) {
        telemetry!(onair: true);
        transmitter.send(&mut pocsag::TestGenerator::new(1125));
        telemetry!(onair: false);
    }
}

impl MessageProvider for SchedulerCore {
    fn next(&mut self, count: usize) -> Option<pocsag::Message> {
        debug!(
            "Remaining time budget: {}",
            self.budget as i32 - count as i32
        );

        if count + 30 > self.budget {
            return None;
        }

        loop {
            match (*self).rx.try_recv() {
                Ok(Command::Message(msg)) => {
                    self.queue.enqueue(msg);
                }
                Ok(Command::SetTimeSlots(slots)) => {
                    self.slots = slots;
                }
                Ok(Command::Stop) |
                Err(TryRecvError::Disconnected) => {
                    self.stop = true;
                    return None;
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
            };
        }

        //let message = self.queue.dequeue();

        //if message.is_some() {
            //status_inc!(calls_tx: 1);
        //}

        //message
        None
    }
}
