use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::str::FromStr;
use std::fmt;

// Returns the time since the unix epoch
fn unix_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

// Returns the time in deciseconds since the unix epoch
fn unix_deciseconds() -> u64 {
    let duration = unix_time();
    let seconds = duration.as_secs();
    let deciseconds = duration.subsec_nanos() as u64 / 100_000_000;
    seconds * 10 + deciseconds
}

#[derive(PartialEq)]
pub struct TimeSlot(usize);

impl TimeSlot {
    pub fn index(&self) -> usize { self.0 }

    pub fn current() -> TimeSlot {
        let time = unix_deciseconds();
        TimeSlot(((time >> 6) & 0b1111) as usize)
    }

    pub fn active(&self) -> bool {
        *self == TimeSlot::current()
    }

    pub fn duration_until(&self) -> Duration {
        let time = unix_deciseconds();
        let start = (time & !((1 << 10) - 1)) + ((self.index() as u64) << 6);
        let start = Duration::new(start/10, (start % 10) as u32 * 100_000_000);
        match start.checked_sub(unix_time()) {
            Some(duration) => duration,
            None => Duration::new(0, 0)
        }
    }
}

pub struct TimeSlots {
    slots: [bool; 16]
}

impl TimeSlots {
    pub fn new() -> TimeSlots {
        TimeSlots { slots: [false; 16] }
    }

    pub fn is_allowed(&self, slot: TimeSlot) -> bool {
        self.slots.get(slot.index()).map(|val| *val).unwrap_or(false)
    }

    pub fn is_current_allowed(&self) -> bool {
        self.is_allowed(TimeSlot::current())
    }

    pub fn next_allowed(&self) -> Option<TimeSlot> {
        let current = TimeSlot::current().index();
        let iterator = self.slots.iter().enumerate().cycle().skip(current);
        for (i, allowed) in iterator.take(self.slots.len()) {
            if *allowed { return Some(TimeSlot(i)); }
        }
        None
    }
}

impl FromStr for TimeSlots {
    type Err = ();

    fn from_str(s: &str) -> Result<TimeSlots, Self::Err> {
        let mut slots = [false; 16];
        for c in s.chars() {
            if let Some(idx) = c.to_digit(16) {
                slots[idx as usize] = true;
            }
        }
        Ok(TimeSlots { slots: slots })
    }
}

impl fmt::Debug for TimeSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimeSlot({:X})", self.0)
    }
}

impl fmt::Debug for TimeSlots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "TimeSlots {{ "));
        for (i, slot) in self.slots.iter().enumerate() {
            if *slot { try!(write!(f, "{:X}", i)); }
        }
        write!(f, " }}")
    }
}

#[test]
pub fn test_timeslots() {
    let slots = TimeSlots::from_str("AC39").unwrap();
    println!("Curr {:?}", TimeSlot::current());
    println!("Next {:?}", slots.next_allowed());
    println!("Until {:?}", slots.next_allowed().unwrap().duration_until());
}
