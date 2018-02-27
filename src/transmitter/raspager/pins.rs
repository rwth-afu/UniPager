pub struct RaspagerPins {
    pub le: usize,
    pub ce: usize,
    pub clk: usize,
    pub sdata: usize,
    pub muxout: usize,
    pub atclk: usize,
    pub atdata: usize,
    pub handshake: usize,
    pub ptt: usize
}

pub const RASPAGER1_PINS: RaspagerPins = RaspagerPins {
    le: 0,
    ce: 7,
    clk: 3,
    sdata: 2,
    muxout: 13,
    atclk: 11,
    atdata: 10,
    handshake: 5,
    ptt: 4
};

pub const RASPAGER2_PINS: RaspagerPins = RaspagerPins {
    le: 9,
    ce: 7,
    clk: 3,
    sdata: 2,
    muxout: 13,
    atclk: 11,
    atdata: 10,
    handshake: 5,
    ptt: 4
};
