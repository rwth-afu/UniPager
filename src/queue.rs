use std::collections::VecDeque;

use crate::message::Message;
use crate::telemetry;

// The number of priorities defines how many seperate queues are beeing used for
// the different priorities. This should match the number of priorities defined
// for the network UniPager is connecting to.
pub const NUM_PRIORITIES: usize = 5;

// The queue. This is a priority queue. It contains subqueues for each priority.
// From the outside it looks just like a normal queue.
pub struct Queue {
    queues: Vec<VecDeque<Message>>,
    sent: [usize; NUM_PRIORITIES]
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            queues: (0..NUM_PRIORITIES).map(|_| VecDeque::new()).collect(),
            sent: [0; NUM_PRIORITIES]
        }
    }

    pub fn enqueue(&mut self, message: Message) {
        self.queues
            .get_mut(message.priority - 1)
            .map(|queue| { queue.push_back(message); })
            .or_else(|| {
                error!("Tried to enqueue message for out of range priority.");
                None
            });
    }

    pub fn dequeue(&mut self) -> Option<Message> {
        for (priority, queue) in self.queues.iter_mut().enumerate().rev() {
            if !queue.is_empty() {
                self.sent[priority] += 1;
                return queue.pop_front();
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.queues.iter().map(&VecDeque::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        !self.queues.iter().any(|queue| !queue.is_empty())
    }

    pub fn telemetry_update(&self, messages: &mut telemetry::Messages) {
        for (priority, queue) in self.queues.iter().enumerate() {
            messages.queued[priority] = queue.len();
        }

        messages.sent = self.sent;
    }
}
