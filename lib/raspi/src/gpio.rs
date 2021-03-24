use std::ptr::{read_volatile, write_volatile};
use std::sync::Arc;

use libc;
use model::Model;
use sysfs_gpio;

pub struct GpioBase(*mut u32);

pub enum Gpio {
    MemGpio {
        base: Arc<GpioBase>,
        pin_mapping: Option<Vec<usize>>,
    },
    SysFsGpio {
        pin_mapping: Option<Vec<usize>>
    },
}

impl Gpio {
    pub fn new() -> Option<Gpio> {
        let model = Model::get();
        let base = model.gpio_base();

        if base.is_none() {
            return Some(Gpio::SysFsGpio {
                pin_mapping: model.pin_mapping()
            });
        }

        let mapped_base = unsafe {
            let mem = "/dev/mem\0".as_ptr() as *const libc::c_char;
            let mem_fd = libc::open(mem, libc::O_RDWR | libc::O_SYNC);
            if mem_fd < 0 {
                return None;
            }

            let mapped_base = libc::mmap(
                0 as *mut libc::c_void,
                0x1000,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                mem_fd,
                base.unwrap() as libc::off_t,
            );

            libc::close(mem_fd);

            if mapped_base == libc::MAP_FAILED {
                return None;
            }

            mapped_base
        };

        Some(Gpio::MemGpio {
            base: Arc::new(GpioBase(mapped_base as *mut u32)),
            pin_mapping: model.pin_mapping(),
        })
    }

    pub fn pin(&self, number: usize, direction: Direction) -> Box<dyn Pin> {
        match self {
            &Gpio::MemGpio { ref base, ref pin_mapping } => {
                let number = pin_mapping.as_ref().and_then(|mapping| {
                    mapping.get(number).map(|num| *num)
                }).unwrap_or(number);
                Box::new(MemGpioPin::new(base.clone(), number, direction))
            }
            &Gpio::SysFsGpio { ref pin_mapping } => {
                let number = pin_mapping.as_ref().and_then(|mapping| {
                    mapping.get(number).map(|num| *num)
                }).unwrap_or(number);
                Box::new(SysFsGpioPin::new(number, direction))
            }
        }
    }
}

impl Drop for GpioBase {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.0 as *mut libc::c_void, 0x1000) };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

pub trait Pin {
    fn set_direction(&mut self, direction: Direction);
    fn set(&self, value: bool);
    fn read(&self) -> bool;

    fn set_high(&self) {
        self.set(true);
    }

    fn set_low(&self) {
        self.set(false);
    }
}

pub struct SysFsGpioPin {
    direction: Direction,
    pin: sysfs_gpio::Pin,
}

impl SysFsGpioPin {
    pub fn new(number: usize, direction: Direction) -> SysFsGpioPin {
        let mut pin = SysFsGpioPin {
            pin: sysfs_gpio::Pin::new(number as u64),
            direction,
        };

        pin.pin.export().expect("Failed to export GPIO pin.");
        pin.set_direction(direction);
        pin
    }
}

impl Pin for SysFsGpioPin {
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;

        let direction = match self.direction {
            Direction::Input => sysfs_gpio::Direction::In,
            Direction::Output => sysfs_gpio::Direction::Out
        };

        self.pin.set_direction(direction).expect("Failed to set GPIO direction.");
    }

    fn set(&self, value: bool) {
        assert_eq!(self.direction, Direction::Output);
        self.pin.set_value(value as u8).ok();
    }

    fn read(&self) -> bool {
        assert_eq!(self.direction, Direction::Input);
        self.pin.get_value().map(|val| val != 0).unwrap_or(false)
    }
}

pub struct MemGpioPin {
    base: Arc<GpioBase>,
    number: usize,
    direction: Direction,
}

impl MemGpioPin {
    pub fn new(base: Arc<GpioBase>, number: usize, direction: Direction) -> MemGpioPin {
        let mut pin = MemGpioPin {
            base,
            number,
            direction,
        };

        pin.set_direction(direction);
        pin
    }
}

impl Pin for MemGpioPin {
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
        let number = self.number;

        match self.direction {
            Direction::Input => unsafe {
                let p = (*self.base).0.offset((number / 10) as isize) as *mut u32;
                *p &= !(0b111 << ((number % 10) * 3));
            },
            Direction::Output => unsafe {
                let p = (*self.base).0.offset((number / 10) as isize) as *mut u32;
                *p &= !(0b111 << ((number % 10) * 3));
                *p |= 0b1 << ((number % 10) * 3);
            }
        };
    }

    fn set(&self, value: bool) {
        assert_eq!(self.direction, Direction::Output);
        if value {
            unsafe {
                let gpio_set = (*self.base).0.offset(7) as *mut u32;
                write_volatile(gpio_set, 1 << self.number);
            }
        } else {
            unsafe {
                let gpio_clr = (*self.base).0.offset(10) as *mut u32;
                write_volatile(gpio_clr, 1 << self.number);
            }
        }
    }

    fn read(&self) -> bool {
        assert_eq!(self.direction, Direction::Input);
        unsafe {
            let gpio_val = read_volatile((*self.base).0.offset(13) as *mut u32);
            (gpio_val & (1 << self.number)) != 0
        }
    }
}

impl Drop for MemGpioPin {
    fn drop(&mut self) {
        match self.direction {
            Direction::Output => {
                self.set_low();
                self.set_direction(Direction::Input);
            }
            Direction::Input => {
                // nothing to clean up
            }
        };
    }
}
