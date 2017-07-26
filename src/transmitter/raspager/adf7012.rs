const XTAL_FREQ: u32 = 4915200;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum OutputDivider {
    Disabled = 0,
    DivideBy2 = 1,
    DivideBy4 = 2,
    DivideBy8 = 3
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
enum Prescaler {
    Scale4_5 = 0,
    Scale8_9 = 1
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum Modulation {
    FSK = 0,
    GFSK = 1,
    ASK = 2,
    OOK = 3
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum MuxOut {
    RegReady = 3,
    DigitalLock = 4
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Adf7012Config {
    output_divider: OutputDivider,
    vco_adjust: u8,
    clock_out_divider: u8,
    xtal_disable: bool,
    xtal_doubler: bool,
    r_divider: u8,
    freq_err_correction: i16,

    prescaler: Prescaler,
    integer_n: u8,
    fractional_n: u16,

    index_counter: u8,
    gfsk_mod_control: u8,
    mod_deviation: u16,
    pa_output_level: u8,
    gaussian_ook: bool,
    mod_control: Modulation,

    pa_bias: u8,
    vco_bias: u8,
    ld_precision: u8,
    muxout: MuxOut,
    vco_disable: bool,
    bleed_down: bool,
    bleed_up: bool,
    charge_pump: u8,
    data_invert: bool,
    clkout_enable: bool,
    pa_enable: bool,
    pll_enable: bool
}

#[allow(dead_code)]
impl Adf7012Config {
    pub fn new() -> Adf7012Config {
        Adf7012Config {
            output_divider: OutputDivider::Disabled,
            vco_adjust: 2,
            clock_out_divider: 1,
            xtal_disable: false,
            xtal_doubler: false,
            r_divider: 1,
            freq_err_correction: -47,

            prescaler: Prescaler::Scale4_5,
            integer_n: 179,
            fractional_n: 128,

            index_counter: 0,
            gfsk_mod_control: 0,
            mod_deviation: 13,
            pa_output_level: 30,
            gaussian_ook: false,
            mod_control: Modulation::FSK,

            pa_bias: 4,
            vco_bias: 1,
            ld_precision: 1,
            muxout: MuxOut::RegReady,
            vco_disable: false,
            bleed_up: false,
            bleed_down: false,
            charge_pump: 3,
            data_invert: true,
            clkout_enable: false,
            pa_enable: false,
            pll_enable: false
        }
    }

    pub fn set_freq(&mut self, freq: u32) {
        let divider = 1 << (self.output_divider as u32);
        let f_pfd = XTAL_FREQ / divider;
        let n = freq / f_pfd;

        let ratio = freq as f64 / f_pfd as f64;
        let rest = ratio - n as f64;

        let m = (rest * 4096.0) as u32;

        self.integer_n = n as u8;
        self.fractional_n = m as u16;
    }

    pub fn set_freq_err_correction(&mut self, value: i16) {
        self.freq_err_correction = value;
    }

    pub fn freq_err_correction(&self) -> u16 {
        if self.freq_err_correction < 0 {
            // Calculate two's complement for a 11-bit number
            (!((-self.freq_err_correction) as u16) & 0b11111111111) + 1
        } else {
            self.freq_err_correction as u16 & 0b1111111111
        }
    }

    pub fn set_muxout(&mut self, muxout: MuxOut) {
        self.muxout = muxout;
    }

    pub fn vco_bias(&self) -> u8 {
        self.vco_bias
    }

    pub fn set_vco_bias(&mut self, value: u8) {
        self.vco_bias = value;
    }

    pub fn vco_adjust(&self) -> u8 {
        self.vco_adjust
    }

    pub fn set_vco_adjust(&mut self, value: u8) {
        self.vco_adjust = value;
    }

    pub fn set_pll_enable(&mut self, value: bool) {
        self.pll_enable = value;
    }

    pub fn set_pa_enable(&mut self, value: bool) {
        self.pa_enable = value;
    }

    pub fn pa_output_level(&mut self) -> u8 {
        self.pa_output_level
    }

    pub fn set_pa_output_level(&mut self, value: u8) {
        if value > 63 {
            self.pa_output_level = 63;
        } else {
            self.pa_output_level = value;
        }
    }

    pub fn r0(&self) -> u32 {
        ((self.output_divider as u32 & 0b11) << 25) |
            ((self.vco_adjust as u32 & 0b11) << 23) |
            ((self.clock_out_divider as u32 & 0b1111) << 19) |
            ((self.xtal_disable as u32) << 18) |
            ((self.xtal_doubler as u32) << 17) |
            ((self.r_divider as u32 & 0b1111) << 13) |
            ((self.freq_err_correction() as u32 & 0b11111111111) << 2)
    }

    pub fn r1(&self) -> u32 {
        1u32 | ((self.prescaler as u32 & 0b1) << 22) |
            ((self.integer_n as u32 & 0b11111111) << 14) |
            ((self.fractional_n as u32 & 0b111111111111) << 2)
    }

    pub fn r2(&self) -> u32 {
        2u32 | ((self.index_counter as u32 & 0b11) << 23) |
            ((self.gfsk_mod_control as u32 & 0b111) << 20) |
            ((self.mod_deviation as u32 & 0b111111111) << 11) |
            ((self.pa_output_level as u32 & 0b111111) << 5) |
            ((self.gaussian_ook as u32 & 0b1) << 4) |
            ((self.mod_control as u32 & 0b11) << 2)
    }

    pub fn r3(&self) -> u32 {
        3u32 | ((self.pa_bias as u32 & 0b111) << 20) |
            ((self.vco_bias as u32 & 0b1111) << 16) |
            ((self.ld_precision as u32 & 0b1) << 15) |
            ((self.muxout as u32 & 0b1111) << 11) |
            ((self.vco_disable as u32 & 0b1) << 10) |
            ((self.bleed_down as u32 & 0b1) << 8) |
            ((self.bleed_up as u32 & 0b1) << 7) |
            ((self.charge_pump as u32 & 0b11) << 6) |
            ((self.data_invert as u32 & 0b1) << 5) |
            ((self.clkout_enable as u32 & 0b1) << 4) |
            ((self.pa_enable as u32 & 0b1) << 3) |
            ((self.pll_enable as u32 & 0b1) << 2)
    }
}

#[test]
pub fn test_freq_err_correction() {
    let mut adf_config = Adf7012Config::new();

    adf_config.set_freq_err_correction(-1024);
    assert_eq!(adf_config.freq_err_correction(), 0b10000000000);

    adf_config.set_freq_err_correction(-1023);
    assert_eq!(adf_config.freq_err_correction(), 0b10000000001);

    adf_config.set_freq_err_correction(-1);
    assert_eq!(adf_config.freq_err_correction(), 0b11111111111);

    adf_config.set_freq_err_correction(0);
    assert_eq!(adf_config.freq_err_correction(), 0);

    adf_config.set_freq_err_correction(1);
    assert_eq!(adf_config.freq_err_correction(), 0b1);

    adf_config.set_freq_err_correction(1023);
    assert_eq!(adf_config.freq_err_correction(), 0b01111111111);
}
