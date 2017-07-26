pub mod generator;
pub mod testgenerator;
pub mod message;
pub mod scheduler;
pub mod timeslots;
pub mod encoding;

pub use self::encoding::Encoding;
pub use self::generator::Generator;
pub use self::message::{Message, MessageFunc, MessageSpeed, MessageType};
pub use self::scheduler::Scheduler;
pub use self::testgenerator::TestGenerator;
pub use self::timeslots::{TimeSlot, TimeSlots};

pub trait MessageProvider {
    fn next(&mut self, count: usize) -> Option<Message>;
}
