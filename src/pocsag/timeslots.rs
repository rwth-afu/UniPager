use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::str::FromStr;
use std::fmt;

// Returns the time since the unix epoch
fn unix_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

// Returns the time in deciseconds since the unix epoch
fn deciseconds(duration: Duration) -> u64 {
    let seconds = duration.as_secs();
    let deciseconds = duration.subsec_nanos() as u64 / 100_000_000;
    seconds * 10 + deciseconds
}

#[derive(PartialEq)]
pub struct TimeSlot(usize);

impl TimeSlot {
    pub fn index(&self) -> usize { self.0 }

    pub fn current() -> TimeSlot {
        let time = deciseconds(unix_time());
        TimeSlot(((time >> 6) & 0b1111) as usize)
    }

    pub fn active(&self) -> bool {
        *self == TimeSlot::current()
    }

    pub fn duration_until(&self) -> Duration {
        let now = unix_time();
        let now_decis = deciseconds(now);

        let current_slot = (now_decis >> 6) & 0b1111;
        let this_slot = self.index() as u64;
        let mut block_start = now_decis & !0b1111111111;

        // if the slot is already over use the next block
        if this_slot == current_slot {
            return Duration::new(0, 0)
        }
        else if this_slot < current_slot {
            block_start += 1 << 10;
        }

        let slot_offset = this_slot << 6;
        let slot_start = block_start + slot_offset;

        let seconds = slot_start/10;
        let nanoseconds = (slot_start % 10) as u32 * 100_000_000;

        let start = Duration::new(seconds, nanoseconds);

        match start.checked_sub(now) {
            Some(duration) => duration,
            None => {
                error!("TimeSlot calculation broken");
                error!("Current Slot: {:X} This Slot: {:X}", current_slot, this_slot);
                error!("Now: {:?}", now);
                error!("Start: {:?}", start);
                if !self.active() {
                    Duration::new(1, 0)
                }
                else {
                    Duration::new(0, 0)
                }
            }
        }
    }
}

#[derive(Serialize, Clone, Copy, PartialEq)]
pub struct TimeSlots([bool; 16]);

impl TimeSlots {
    pub fn new() -> TimeSlots {
        TimeSlots([false; 16])
    }

    pub fn is_allowed(&self, slot: TimeSlot) -> bool {
        self.0.get(slot.index()).map(|val| *val).unwrap_or(false)
    }

    pub fn is_current_allowed(&self) -> bool {
        self.is_allowed(TimeSlot::current())
    }

    pub fn next_allowed(&self) -> Option<TimeSlot> {
        let current = TimeSlot::current().index();
        let iterator = self.0.iter().enumerate().cycle().skip(current);
        for (i, allowed) in iterator.take(self.0.len()) {
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
        Ok(TimeSlots(slots))
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
        for (i, slot) in self.0.iter().enumerate() {
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
