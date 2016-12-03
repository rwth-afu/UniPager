use sysfs_gpio::{Pin, Direction};
use std::{thread, time};
use std::ops::DerefMut;

use generator::Generator;
use raspager::adf7012::{Adf7012Config, MuxOut};

const RF_FREQ: u32 = 439987500;

#[inline]
fn delay_us(micros: u32) {
    thread::sleep(time::Duration::new(0, micros*1000));
}

#[inline]
fn delay_ms(millis: u64) {
    thread::sleep(time::Duration::from_millis(millis));
}

pub struct Transmitter {
    le: Pin,
    ce: Pin,
    clk: Pin,
    sdata: Pin,
    muxout: Pin,
    txclk: Pin,
    clkout: Pin,
    atclk: Pin,
    atdata: Pin,
    handshake: Pin,
    ptt: Pin,
    config: Adf7012Config
}

impl Transmitter  {
    pub fn new() -> Transmitter {
        let tx = Transmitter {
            le: Pin::new(17),
            ce: Pin::new(4),
            clk: Pin::new(22),
            sdata: Pin::new(27),
            muxout: Pin::new(9),
            txclk: Pin::new(11),
            clkout: Pin::new(10),
            atclk: Pin::new(7),
            atdata: Pin::new(8),
            handshake: Pin::new(24),
            ptt: Pin::new(23),
            config: Adf7012Config::new()
        };

        for pin in vec![&tx.le, &tx.ce, &tx.clk, &tx.sdata, &tx.clkout, &tx.atdata, &tx.atclk] {
            pin.export().expect("Unable to export pin");
            pin.set_direction(Direction::Low).expect("Unable to set pin as low output");
        }

        for pin in vec![&tx.muxout, &tx.txclk, &tx.handshake, &tx.ptt] {
            pin.export().expect("Unable to export pin");
            pin.set_direction(Direction::In).expect("Unable to set pin as input");
        }

        tx
    }

    pub fn run(&mut self) {
        info!("run");
        self.reset();
        self.config.set_freq(RF_FREQ);
        self.write_config();
    }

    fn ptt_on(&mut self) -> bool {
        self.ce.set_value(1).unwrap();
        self.config.set_pa_enable(true);
        self.config.set_pa_output_level(63);
        self.config.set_muxout(MuxOut::RegReady);
        self.write_config();
        delay_ms(100);

        if self.muxout.get_value().unwrap() != 0 {
            if self.lock_pll() {
                self.config.set_pa_enable(true);
                self.config.set_pa_output_level(63);
                self.write_config();
                delay_ms(50);
                true
            }
            else {
                error!("PLL locking failed");
                self.ptt_off();
                false
            }
        }
        else {
            error!("ADF7012 not ready");
            false
        }
    }

    fn ptt_off(&mut self) {
        self.config.set_pa_enable(false);
        self.config.set_pa_output_level(0);
        self.write_config();

        delay_ms(100);
        self.ce.set_value(0).unwrap();
    }

    fn lock_pll(&mut self) -> bool {
        let mut adj = self.config.vco_adjust();
        let mut bias = self.config.vco_bias();

        self.config.set_pll_enable(true);
        self.config.set_muxout(MuxOut::DigitalLock);

        self.write_config();
        delay_ms(500);

        while self.muxout.get_value().unwrap() == 0 {
            info!("Trying to lock {} {}", adj, bias);
            self.config.set_vco_adjust(adj);
            self.config.set_vco_bias(bias);
            self.write_config();
            delay_ms(500);

            bias += 1;

            if bias > 13 {
                bias = 1;
                adj += 1;

                if adj > 3 {
                    self.config.set_vco_adjust(0);
                    self.config.set_vco_bias(0);
                    return false;
                }
            }
        }

        info!("PLL locked");
        true
    }

    fn write_config(&mut self) {
        info!("write config: {:?}", self.config);
        let registers = vec![self.config.r0(), self.config.r1(),
                             self.config.r2(), self.config.r3()];

        for register in registers {
            self.write_register(register);
        }
    }

    fn write_register(&mut self, register: u32) {
        self.clk.set_value(0).unwrap();
        delay_us(2);
        self.le.set_value(0).unwrap();
        delay_us(10);

        for i in (0..32).rev() {
            let bit = if (register & (1 << i)) != 0 { 1 } else { 0 };
            self.sdata.set_value(bit).unwrap();
            delay_us(10);
            self.clk.set_value(1).unwrap();
            delay_us(30);
            self.clk.set_value(0).unwrap();
            delay_us(30);
        }

        delay_us(10);
        self.le.set_value(1).unwrap();
    }

    fn reset(&mut self) {
        self.ce.set_value(0).unwrap();
        self.le.set_value(1).unwrap();
        self.clk.set_value(1).unwrap();
        self.sdata.set_value(1).unwrap();
        delay_ms(5);
        self.ce.set_value(1).unwrap();
        delay_ms(100);
    }
}

impl ::transmitter::Transmitter for Transmitter {
    fn send(&mut self, gen: Generator) {
        info!("Sending data...");

        if !self.ptt_on() {
            return;
        }

        for byte in gen {
            for i in (0..32).rev() {
                while self.handshake.get_value().unwrap() == 0 {
                    delay_us(100);
                }

                let bit = if (byte & (1 << i)) != 0 { 1 } else { 0 };
                self.atdata.set_value(bit).unwrap();

                delay_us(20);
                self.atclk.set_value(1).unwrap();
                delay_us(100);
                self.atclk.set_value(0).unwrap();
                delay_us(50);
            }
        }
        self.ptt_off();
        delay_ms(200);

        info!("Data sent.");
    }
}
