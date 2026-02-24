use std::vec;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const TOTAL_PIXELS: usize = SCREEN_HEIGHT * SCREEN_WIDTH;
pub const MEMORY_SIZE: usize = 4096;
pub const STACK_SIZE: usize = 16;
pub const TOTAL_REGISTER: usize = 16;
pub const TOTAL_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

pub struct Emulator {
    pc: u16,
    memory: [u8; MEMORY_SIZE],
    register: [u8; TOTAL_REGISTER],
    i_reg: u16,
    stack: Vec<u16>,
    display: [bool; TOTAL_PIXELS],
    delay_timer: u8,
    sound_timer: u8,
    keys: [bool; TOTAL_KEYS],
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            memory: [0; MEMORY_SIZE],
            register: [0; TOTAL_REGISTER],
            i_reg: 0,
            stack: Vec::with_capacity(STACK_SIZE),
            display: [false; TOTAL_PIXELS],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; TOTAL_KEYS],
        }
    }
}
