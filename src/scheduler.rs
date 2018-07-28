use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use futures::{Future, Stream};
use futures::prelude::Async;
use tokio::timer::Deadline;

use config::Config;
use event::{self, Event, EventHandler, EventReceiver};
use message::{Message, MessageProvider};
use pocsag;
use queue::Queue;
use telemetry;
use timeslots::{TimeSlot, TimeSlots};
use transmitter::{self, Transmitter};

#[derive(Clone)]
pub struct Scheduler {
    scheduler: Arc<Mutex<SchedulerCore>>
}

struct SchedulerCore {
    rx: EventReceiver,
    slots: TimeSlots,
    queue: Queue,
    budget: usize
}

impl Scheduler {
    pub fn new(event_handler: EventHandler) -> Scheduler {
        use std::str::FromStr;

        let (tx, rx) = event::channel();

        event_handler.publish(Event::RegisterScheduler(tx));

        let core = SchedulerCore {
            rx: rx,
            slots: TimeSlots::from_str("0123456789ABCDEF").unwrap(),
            queue: Queue::new(),
            budget: 0
        };

        Scheduler { scheduler: Arc::new(Mutex::new(core)) }
    }

    pub fn start(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
        thread::spawn(move || {
            let transmitter = transmitter::from_config(&config);
            scheduler.scheduler.lock().unwrap().run(transmitter);
        })
    }
}

impl SchedulerCore {
    pub fn run(&mut self, mut transmitter: Box<Transmitter>) {
        info!("Scheduler started.");

        loop {
            while self.queue.is_empty() {
                self.process_next_event();
            }

            self.wait_for_next_timeslot();

            debug!("Available time budget: {}", self.budget);

            let message = self.queue.dequeue().unwrap();
            telemetry!(onair: true);
            transmitter.send(&mut *message.generator(self));
            telemetry!(onair: false);
        }
    }

    fn wait_for_next_timeslot(&mut self) {
        loop {
            if self.slots.is_current_allowed() &&
                self.slots.calculate_budget() > 30
            {
                return;
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
            }
            Event::TimeslotsUpdate(slots) => {
                self.slots = slots;
            }
            _ => {}
        }

    }

    fn recv_event(&mut self) -> Option<Event> {
        let rx_next = self.rx.by_ref().into_future();

        match rx_next.wait()
        {
            Ok((event, _)) => event,
            _ => None,
        }
    }

    fn recv_event_timeout(&mut self, duration: Duration) -> Option<Event> {
        let deadline = Instant::now() + duration;
        let rx_next = Deadline::new(self.rx.by_ref().into_future(), deadline);

        match rx_next.wait()
        {
            Ok((event, _)) => event,
            _ => None,
        }
    }
}

impl MessageProvider for SchedulerCore {
    fn next(&mut self, count: usize) -> Option<Message> {
        debug!(
            "Remaining time budget: {}",
            self.budget as i32 - count as i32
        );

        if count + 30 > self.budget {
            return None;
        }

        loop {
            match self.rx.poll()
            {
                Ok(Async::Ready(Some(event))) => {
                    self.process_event(event);
                }
                Ok(Async::Ready(None)) |
                Ok(Async::NotReady) |
                Err(_) => {
                    break;
                }
            };
        }

        self.queue.dequeue()
    }
}
