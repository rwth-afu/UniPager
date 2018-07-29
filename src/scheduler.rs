use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use config::Config;
use event::{Event, EventHandler};
use message::{Message, MessageProvider};
use queue::Queue;
use timeslots::TimeSlots;
use transmitter::{self, Transmitter};

struct Scheduler {
    rx: Receiver<Event>,
    slots: TimeSlots,
    queue: Queue,
    budget: usize
}

pub fn start(config: Config, event_handler: EventHandler) {
    use std::str::FromStr;

    let (tx, rx) = mpsc::channel();

    event_handler.publish(Event::RegisterScheduler(tx));

    let mut scheduler = Scheduler {
        rx: rx,
        slots: TimeSlots::from_str("ACF").unwrap(),
        queue: Queue::new(),
        budget: 0
    };

    thread::spawn(move || {
        let transmitter = transmitter::from_config(&config);
        scheduler.run(transmitter);
    });
}

impl Scheduler {
    pub fn run(&mut self, mut transmitter: Box<Transmitter>) {
        info!("Scheduler started.");

        loop {
            while self.queue.is_empty() {
                info!("Queue Empty, waiting for events.");
                self.process_next_event();
            }

            info!("Queue not empty, waiting for next Timeslot.");
            self.wait_for_next_timeslot();

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
                Err(_) => {
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
