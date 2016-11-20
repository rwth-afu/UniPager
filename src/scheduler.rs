use std::sync::{Arc, RwLock, Mutex};
use std::collections::VecDeque;

use message::Message;

#[derive(Clone)]
pub struct Scheduler {
    queue: Arc<Mutex<VecDeque<Message>>>,
    state: Arc<RwLock<SchedulerState>>
}

struct SchedulerState {
    pub time: u16
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            state: Arc::new(RwLock::new(SchedulerState { time: 0 }))
        }
    }

    pub fn run(&self) {
        loop {

        }
    }

    pub fn enqueue(&self, msg: Message) {
        info!("Enqueue new message: {:?}", msg);
        self.queue.lock().unwrap().push_back(msg);
    }

    pub fn time(&self) -> u16 {
        self.state.read().unwrap().time
    }

    pub fn correct_time(&self, correction: u16) {
        let time = &(self.state.get_mut().unwrap().time);
        *time = *time + correction;
    }
}
