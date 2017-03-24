pub mod generator;
pub mod message;
pub mod scheduler;
pub mod timeslots;
pub mod encoding;

pub use self::generator::Generator;
pub use self::message::{Message, MessageSpeed, MessageType, MessageFunc};
pub use self::scheduler::Scheduler;
pub use self::timeslots::{TimeSlots, TimeSlot};
pub use self::encoding::Encoding;

pub trait MessageProvider {
    fn next(&mut self) -> Option<Message>;
}
