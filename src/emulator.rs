use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const TOTAL_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const MEMORY_SIZE: usize = 4096;
pub const STACK_SIZE: usize = 16;
pub const TOTAL_REGISTER: usize = 16;
pub const TOTAL_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;
pub const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

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
        let mut new_emu = Self {
            pc: START_ADDR,
            memory: [0; MEMORY_SIZE],
            register: [0; TOTAL_REGISTER],
            i_reg: 0,
            stack: Vec::with_capacity(STACK_SIZE),
            display: [false; TOTAL_PIXELS],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; TOTAL_KEYS],
        };
        new_emu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.memory = [0; MEMORY_SIZE];
        self.register = [0; TOTAL_REGISTER];
        self.i_reg = 0;
        self.stack = Vec::with_capacity(STACK_SIZE);
        self.display = [false; TOTAL_PIXELS];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keys = [false; TOTAL_KEYS];
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn fetch(&mut self) -> u16 {
        let msb = self.memory[self.pc as usize] as u16;
        let lsb = self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        (msb << 8) | lsb
    }

    pub fn cpu_cycle(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    pub fn tick_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: beep
            }
            self.sound_timer -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load_game(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => self.display = [false; TOTAL_PIXELS],
            (0, 0, 0xE, 0xE) => {
                if let Some(ret) = self.stack.pop() {
                    self.pc = ret;
                } else {
                    eprintln!("Stack underflow!");
                }
            }
            (1, _, _, _) => self.pc = opcode & 0x0FFF,
            (2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = opcode & 0x0FFF;
            }
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.register[x] == nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.register[x] != nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.register[x] == self.register[y] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => self.register[digit2 as usize] = (opcode & 0xFF) as u8,
            (7, _, _, _) => {
                let x = digit2 as usize;
                self.register[x] = self.register[x].wrapping_add((opcode & 0xFF) as u8);
            }
            (8, _, _, 0) => self.register[digit2 as usize] = self.register[digit3 as usize],
            (8, _, _, 1) => self.register[digit2 as usize] |= self.register[digit3 as usize],
            (8, _, _, 2) => self.register[digit2 as usize] &= self.register[digit3 as usize],
            (8, _, _, 3) => self.register[digit2 as usize] ^= self.register[digit3 as usize],
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (val, carry) = self.register[x].overflowing_add(self.register[y]);
                self.register[x] = val;
                self.register[0xF] = if carry { 1 } else { 0 };
            }
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (val, borrow) = self.register[x].overflowing_sub(self.register[y]);
                self.register[x] = val;
                self.register[0xF] = if borrow { 0 } else { 1 };
            }
            (8, _, _, 6) => {
                let x = digit2 as usize;
                self.register[0xF] = self.register[x] & 1;
                self.register[x] >>= 1;
            }
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (val, borrow) = self.register[y].overflowing_sub(self.register[x]);
                self.register[x] = val;
                self.register[0xF] = if borrow { 0 } else { 1 };
            }
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                self.register[0xF] = (self.register[x] >> 7) & 1;
                self.register[x] <<= 1;
            }
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.register[x] != self.register[y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => self.i_reg = opcode & 0x0FFF,
            (0xB, _, _, _) => self.pc = self.register[0] as u16 + (opcode & 0x0FFF),
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.register[x] = random::<u8>() & nn;
            }
            (0xD, _, _, _) => {
                let x_coord = self.register[digit2 as usize] as u16;
                let y_coord = self.register[digit3 as usize] as u16;
                let rows = digit4;
                let mut flipped = false;

                for y_line in 0..rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.memory[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = ((x_coord + x_line) as usize) % SCREEN_WIDTH;
                            let y = ((y_coord + y_line) as usize) % SCREEN_HEIGHT;
                            let idx = x + y * SCREEN_WIDTH;

                            if self.display[idx] {
                                flipped = true;
                            }
                            self.display[idx] ^= true;
                        }
                    }
                }

                self.register[0xF] = if flipped { 1 } else { 0 };
            }
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                if self.keys[self.register[x] as usize] {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                if !self.keys[self.register[x] as usize] {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => self.register[digit2 as usize] = self.delay_timer,
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..TOTAL_KEYS {
                    if self.keys[i] {
                        self.register[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => self.delay_timer = self.register[digit2 as usize],
            (0xF, _, 1, 8) => self.sound_timer = self.register[digit2 as usize],
            (0xF, _, 1, 0xE) => {
                self.i_reg = self
                    .i_reg
                    .wrapping_add(self.register[digit2 as usize] as u16)
            }
            (0xF, _, 2, 9) => self.i_reg = (self.register[digit2 as usize] as u16) * 5,
            (0xF, _, 3, 3) => {
                let i = self.i_reg as usize;
                let vx = self.register[digit2 as usize];
                self.memory[i] = vx / 100;
                self.memory[i + 1] = (vx / 10) % 10;
                self.memory[i + 2] = vx % 10;
            }
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for offset in 0..=x {
                    self.memory[i + offset] = self.register[offset];
                }
            }
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for offset in 0..=x {
                    self.register[offset] = self.memory[i + offset];
                }
            }
            _ => unimplemented!("Unimplemented opcode: 0x{:04X}", opcode),
        }
    }
}
