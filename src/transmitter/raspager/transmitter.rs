use std::{thread, time};

use config::Config;
use raspi::{Direction, Gpio, Model, Pin};
use transmitter::Transmitter;
use transmitter::raspager::adf7012::{Adf7012Config, MuxOut};
use transmitter::raspager::pins::RaspagerPins;

#[inline]
fn delay_us(micros: u32) {
    thread::sleep(time::Duration::new(0, micros * 1000));
}

#[inline]
fn delay_ms(millis: u64) {
    thread::sleep(time::Duration::from_millis(millis));
}

pub struct RaspagerTransmitter {
    le: Pin,
    ce: Pin,
    clk: Pin,
    sdata: Pin,
    muxout: Pin,
    atclk: Pin,
    atdata: Pin,
    handshake: Pin,
    ptt: Pin,
    config: Adf7012Config,
    output_level: u8
}

impl RaspagerTransmitter {
    pub fn new(config: &Config, pins: RaspagerPins) -> RaspagerTransmitter {
        info!("Initializing RasPager transmitter...");
        info!("Detected {}", Model::get());
        let gpio = Gpio::new().expect("Failed to map GPIO");

        let mut tx = RaspagerTransmitter {
            le: gpio.pin(pins.le, Direction::Output),
            ce: gpio.pin(pins.ce, Direction::Output),
            clk: gpio.pin(pins.clk, Direction::Output),
            sdata: gpio.pin(pins.sdata, Direction::Output),
            muxout: gpio.pin(pins.muxout, Direction::Input),
            atclk: gpio.pin(pins.atclk, Direction::Output),
            atdata: gpio.pin(pins.atdata, Direction::Output),
            handshake: gpio.pin(pins.handshake, Direction::Input),
            ptt: gpio.pin(pins.ptt, Direction::Input),
            config: Adf7012Config::new(),
            output_level: config.raspager.pa_output_level
        };

        tx.reset();
        tx.config.set_freq_err_correction(config.raspager.freq_corr);
        tx.config.set_freq(config.raspager.freq);
        tx.config.set_mod_deviation(config.raspager.mod_deviation);
        tx.write_config();

        tx
    }

    fn ptt_on(&mut self) -> bool {
        self.ce.set_high();
        self.config.set_pa_enable(false);
        self.config.set_pa_output_level(0);
        self.config.set_muxout(MuxOut::RegReady);
        self.write_config();
        delay_ms(100);

        if self.muxout.read() {
            if self.lock_pll() {
                self.config.set_pa_enable(true);
                self.config.set_pa_output_level(self.output_level);
                self.write_config();
                delay_ms(50);
                true
            } else {
                error!("PLL locking failed");
                self.ptt_off();
                false
            }
        } else {
            debug!("ADF7012 not ready");
            false
        }
    }

    fn ptt_off(&mut self) {
        while self.ptt.read() {
            debug!("PTT still high");
            delay_ms(100);
        }

        self.config.set_pa_enable(false);
        self.config.set_pa_output_level(0);
        self.write_config();

        delay_ms(100);
        self.ce.set_low();
    }

    fn lock_pll(&mut self) -> bool {
        let mut adj = self.config.vco_adjust();
        let mut bias = self.config.vco_bias();

        self.config.set_pll_enable(true);
        self.config.set_muxout(MuxOut::DigitalLock);

        self.write_config();
        delay_ms(500);

        while !self.muxout.read() {
            debug!("Trying to lock {} {}", adj, bias);
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

        debug!("PLL locked");
        true
    }

    fn write_config(&mut self) {
        debug!("write config: {:?}", self.config);
        let registers = vec![
            self.config.r0(),
            self.config.r1(),
            self.config.r2(),
            self.config.r3(),
        ];

        for register in registers {
            self.write_register(register);
        }
    }

    fn write_register(&mut self, register: u32) {
        self.clk.set_low();
        delay_us(2);
        self.le.set_low();
        delay_us(10);

        for i in (0..32).rev() {
            let bit = (register & (1 << i)) != 0;
            self.sdata.set(bit);
            delay_us(10);
            self.clk.set_high();
            delay_us(30);
            self.clk.set_low();
            delay_us(30);
        }

        delay_us(10);
        self.le.set_high();
    }

    fn reset(&mut self) {
        self.ce.set_low();
        self.le.set_high();
        self.clk.set_high();
        self.sdata.set_high();
        delay_ms(5);
        self.ce.set_high();
        delay_ms(100);
    }
}

impl Transmitter for RaspagerTransmitter {
    fn send(&mut self, gen: &mut Iterator<Item = u32>) {
        // try multiple times until the PLL is locked
        let mut pll_locked = false;
        for _ in 0..5 {
            pll_locked = self.ptt_on();
            if pll_locked {
                break;
            }
        }

        if !pll_locked {
            error!("Could not transmit message: PLL locking failed");
            self.ptt_off();
            delay_ms(200);
            return;
        }

        for word in gen {
            for i in (0..32).rev() {
                while !self.handshake.read() {
                    debug!("ATmega Buffer full");
                    delay_us(100);
                }

                let bit = (word & (1 << i)) != 0;
                self.atdata.set(bit);

                delay_us(20);
                self.atclk.set_high();
                delay_us(100);
                self.atclk.set_low();
                delay_us(50);
            }
        }

        self.atdata.set_low();

        self.ptt_off();
        delay_ms(200);
    }
}
