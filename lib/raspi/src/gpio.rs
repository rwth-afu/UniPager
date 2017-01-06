use std::sync::Arc;
use std::intrinsics::{offset, volatile_load, volatile_store};
use libc;
use model::Model;

pub struct GpioBase(*mut u32);

pub struct Gpio {
    pub base: Arc<GpioBase>,
    pin_mapping: Vec<usize>
}

impl Gpio {
    pub fn new() -> Option<Gpio> {
        let model = Model::get();
        let base = model.gpio_base();

        if base.is_none() { return None };

        let mapped_base = unsafe {
            let mem = "/dev/mem\0".as_ptr() as *const libc::c_char;
            let mem_fd = libc::open(mem, libc::O_RDWR|libc::O_SYNC);
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

            if mapped_base == libc::MAP_FAILED {
                return None;
            }

            mapped_base
        };

        Some(Gpio {
            base: Arc::new(GpioBase(mapped_base as *mut u32)),
            pin_mapping: model.pin_mapping()
        })
    }

    pub fn pin(&self, number: usize, direction: Direction) -> Pin {
        let number = self.pin_mapping.get(number).unwrap();
        Pin::new(self.base.clone(), *number, direction)
    }
}

impl Drop for GpioBase {
    fn drop(&mut self) {
        let res = unsafe { libc::munmap(self.0 as *mut libc::c_void, 0x1000) };
    }
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Input,
    Output
}

pub struct Pin {
    base: Arc<GpioBase>,
    number: usize,
    direction: Direction
}

impl Pin {
    pub fn new(base: Arc<GpioBase>, number: usize, direction: Direction) -> Pin {
        match direction {
            Direction::Input => unsafe {
                let p = offset((*base).0, (number/10) as isize) as *mut u32;
                *p &= !(0b111 << ((number % 10) * 3));
            },
            Direction::Output => unsafe {
                let p = offset((*base).0, (number/10) as isize) as *mut u32;
                *p &= !(0b111 << ((number % 10) * 3));
                *p |= 0b1 << ((number % 10) * 3);
            }
        }

        Pin {
            base: base,
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
                let gpio_set = offset((*self.base).0, 7) as *mut u32;
                volatile_store(gpio_set, 1 << self.number);
            }
        }
        else {
            unsafe {
                let gpio_clr = offset((*self.base).0, 10) as *mut u32;
                volatile_store(gpio_clr, 1 << self.number);
            }
        }
    }

    pub fn read(&self) -> bool {
        assert_eq!(self.direction, Direction::Input);
        unsafe {
            let gpio_val = volatile_load(offset((*self.base).0, 13) as *mut u32);
            (gpio_val & (1 << self.number)) != 0
        }
    }
}
