use libc;
use std::intrinsics::{offset, volatile_load, volatile_store};
use model::Model;

#[derive(Clone, Copy)]
pub struct Gpio {
    pub base: *mut u32
}

impl Gpio {
    pub fn new() -> Option<Gpio> {
        let model = Model::get();
        let base = model.gpio_base();

        if base.is_none() { return None };

        let mapped_base = unsafe {
            let mem_fd = libc::open("/dev/mem\0".as_ptr() as *const libc::c_char,
                                    libc::O_RDWR|libc::O_SYNC);

            if mem_fd < 0 { return None; }

            let mapped_base = libc::mmap(
              0 as *mut libc::c_void,
              0x1000,
              libc::PROT_READ|libc::PROT_WRITE,
              libc::MAP_SHARED,
              mem_fd,
              base.unwrap() as libc::off_t
            );

            libc::close(mem_fd);

            if mapped_base == libc::MAP_FAILED { return None; }

            mapped_base
        };

        Some(Gpio {
            base: mapped_base as *mut u32
        })
    }

    pub fn pin(&self, number: u8, direction: Direction) -> Pin {
        Pin::new(*self, number, direction)
    }
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Input,
    Output
}

pub struct Pin {
    gpio: Gpio,
    number: u8,
    direction: Direction
}

impl Pin {
    pub fn new(gpio: Gpio, number: u8, direction: Direction) -> Pin {
        match direction {
            Direction::Input => unsafe {
                let p = offset(gpio.base, (number/10) as isize) as *mut u32;
                *p &= !(0b111 << ((number % 10) * 3));
            },
            Direction::Output => unsafe {
                let p = offset(gpio.base, (number/10) as isize) as *mut u32;
                *p &= !(0b111 << ((number % 10) * 3));
                *p |= 0b1 << ((number % 10) * 3);
            }
        }

        Pin {
            gpio: gpio,
            number: number,
            direction: direction
        }
    }

    pub fn set_high(&self) {
        self.set(true);
    }

    pub fn set_low(&self) {
        self.set(false);
    }

    pub fn set(&self, value: bool) {
        assert_eq!(self.direction, Direction::Output);
        if value {
            unsafe {
                let gpio_set = offset(self.gpio.base, 7) as *mut u32;
                volatile_store(gpio_set, 1 << self.number);
            }
        }
        else {
            unsafe {
                let gpio_clr = offset(self.gpio.base, 10) as *mut u32;
                volatile_store(gpio_clr, 1 << self.number);
            }
        }
    }

    pub fn read(&self) -> bool {
        assert_eq!(self.direction, Direction::Input);
        unsafe {
            let gpio_val = volatile_load(offset(self.gpio.base, 13) as *mut u32);
            (gpio_val & (1 << self.number)) != 0
        }
    }
}
