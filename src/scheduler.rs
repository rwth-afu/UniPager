use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::event::{Event, EventHandler};
use crate::message::{Message, MessageProvider};
use crate::pocsag::TestGenerator;
use crate::queue::Queue;
use crate::timeslots::TimeSlots;
use crate::transmitter::{self, Transmitter};

struct Scheduler {
    config: Config,
    rx: Receiver<Event>,
    slots: TimeSlots,
    queue: Queue,
    budget: usize,
    test: bool,
    stop: bool,
    restart: bool,
}

pub fn start(config: Config, event_handler: EventHandler) {
    let (tx, rx) = mpsc::channel();
    event_handler.publish(Event::RegisterScheduler(tx));

    thread::spawn(move || {
        let mut scheduler = Scheduler::new(config, rx);
        scheduler.start();
    });
}

impl Scheduler {
    pub fn new(config: Config, rx: Receiver<Event>) -> Scheduler {
        Scheduler {
            config,
            rx,
            slots: TimeSlots::new(),
            queue: Queue::new(),
            budget: 0,
            test: false,
            stop: false,
            restart: true,
        }
    }

    pub fn start(&mut self) {
        loop {
            let transmitter = transmitter::from_config(&self.config);
            if self.test {
                self.test(transmitter);
                self.test = false;
            } else {
                self.run(transmitter);
            }

            if !self.restart {
                info!("Shutting down the scheduler...");
                return;
            } else {
                info!("Restarting the scheduler...");
                self.stop = false;
            }
        }
    }

    pub fn run(&mut self, mut transmitter: Box<dyn Transmitter>) {
        info!("Scheduler started.");

        loop {
            while self.queue.is_empty() {
                info!("Queue Empty, waiting for events.");
                self.process_next_event();
                if self.stop { return; }
            }

            info!("Queue not empty, waiting for next Timeslot. {} messages waiting.", self.queue.len());
            self.wait_for_next_timeslot();
            if self.stop { return; }

            info!("Available time budget: {}", self.budget);
            let message = self.queue.dequeue().unwrap();

            telemetry_update!(messages: |m| {
                self.queue.telemetry_update(m);
            });

            telemetry!(onair: true);
            transmitter.send(&mut *message.generator(self));
            telemetry!(onair: false);
        }
    }

    pub fn test(&mut self, mut transmitter: Box<dyn Transmitter>) {
        telemetry!(onair: true);
        transmitter.send(&mut TestGenerator::new(1125));
        telemetry!(onair: false);
    }

    fn wait_for_next_timeslot(&mut self) {
        loop {
            if self.slots.is_current_allowed() {
                self.budget = self.slots.calculate_budget();
                if self.budget > 30 {
                    return;
                }
            }

            let event = self.slots
                .next_allowed()
                .map(|next_slot| next_slot.duration_until())
                .map(|duration| self.recv_event_timeout(duration))
                .unwrap_or_else(|| self.recv_event());

            match event
            {
                Some(event) => self.process_event(event),
                None => return,
            }

            if self.stop { return; }
        }
    }

    fn process_next_event(&mut self) {
        if let Some(event) = self.recv_event() {
            self.process_event(event);
        }
    }

    fn process_event(&mut self, event: Event) {
        match event
        {
            Event::MessageReceived(msg) => {
                self.queue.enqueue(msg);
                telemetry_update!(messages: |m| {
                    self.queue.telemetry_update(m);
                });
            }
            Event::TimeslotsUpdate(slots) => {
                self.slots = slots;
                telemetry!(timeslots: slots);
            }
            Event::ConfigUpdate(config) => {
                self.config = config;
                self.stop = true;
                self.restart = true;
            }
            Event::Test => {
                self.test = true;
                self.stop = true;
                self.restart = true;
            }
            Event::Restart => {
                self.stop = true;
                self.restart = true;
            }
            Event::Shutdown => {
                self.stop = true;
                self.restart = false;
            }
            _ => {}
        }
    }

    fn recv_event(&mut self) -> Option<Event> {
        self.rx.recv().ok()
    }

    fn recv_event_timeout(&mut self, duration: Duration) -> Option<Event> {
        self.rx.recv_timeout(duration).ok()
    }
}

impl MessageProvider for Scheduler {
    fn next(&mut self, count: usize) -> Option<Message> {
        debug!(
            "Remaining time budget: {}",
            self.budget as i32 - count as i32
        );

        if count + 30 > self.budget {
            return None;
        }

        loop {
            match self.rx.try_recv()
            {
                Ok(event) => {
                    self.process_event(event);
                }
                Err(TryRecvError::Disconnected) => {
                    self.stop = true;
                    return None;
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
            };
        }

        let message = self.queue.dequeue();

        telemetry_update!(messages: |m| {
            self.queue.telemetry_update(m);
        });

        message
    }
}
