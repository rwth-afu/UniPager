use std::str::FromStr;
use std::fmt;

pub struct TimeSlots {
    slots: [bool; 16]
}

impl TimeSlots {
    pub fn new() -> TimeSlots {
        TimeSlots { slots: [false; 16] }
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

impl fmt::Debug for TimeSlots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "TimeSlots {{ "));
        for (i, slot) in self.slots.iter().enumerate() {
            if *slot { try!(write!(f, "{:X}", i)); }
        }
        write!(f, " }}")
    }
}
