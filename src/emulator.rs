use std::vec;
use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const TOTAL_PIXELS: usize = SCREEN_HEIGHT * SCREEN_WIDTH;
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

    // Opcode is 2 bytes long, so we need to access 2 indices from the memory array
    fn fetch(&mut self) -> u16 {
        let msb = self.memory[self.pc as usize] as u16; // Most significant Byte
        let lsb = self.memory[(self.pc + 1) as usize] as u16; // Least significant Byte
        self.pc += 2;
        let opcode = (msb << 8) | lsb;
        opcode
    }

    pub fn cpu_cycle(&mut self) {
        // Fetch
        let opcode = self.fetch();
        // Decode & Execute
        self.execute(opcode);
    }

    pub fn tick_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                //TODO: beep
            }
            self.sound_timer -= 1;
        }
    }

    fn execute(&mut self, opcode: u16) {
    // Split the opcode into 4 nibbles (hex digits)
    let digit1 = (opcode & 0xF000) >> 12;
    let digit2 = (opcode & 0x0F00) >> 8;
    let digit3 = (opcode & 0x00F0) >> 4;
    let digit4 = opcode & 0x000F;

    match (digit1, digit2, digit3, digit4) {
        // 0x0000: Do nothing
        (0, 0, 0, 0) => return,

        // 0x00E0: Clear the display
        (0, 0, 0xE, 0) => {
            self.display = [false; TOTAL_PIXELS];
        }

        // 0x00EE: Return from subroutine
        (0, 0, 0xE, 0xE) => {
            if let Some(return_addr) = self.stack.pop() {
                self.pc = return_addr;
            } else {
                eprintln!("Stack underflow!");
            }
        }

        // 0x1NNN: Jump to address NNN
        (1, _, _, _) => {
            let nnn = opcode & 0x0FFF;
            self.pc = nnn;
        }

        // 0x2NNN: Call subroutine at NNN
        (2, _, _, _) => {
            let nnn = opcode & 0x0FFF;
            self.stack.push(self.pc);
            self.pc = nnn;
        }

        // 0x3XNN: Skip next instruction if Vx == NN
        (3, _, _, _) => {
            let x = digit2 as usize;
            let nn = (opcode & 0x00FF) as u8;
            if self.register[x] == nn {
                self.pc += 2;
            }
        }

        // 0x4XNN: Skip next instruction if Vx != NN
        (4, _, _, _) => {
            let x = digit2 as usize;
            let nn = (opcode & 0x00FF) as u8;
            if self.register[x] != nn {
                self.pc += 2;
            }
        }

        // 0x5XY0: Skip next instruction if Vx == Vy
        (5, _, _, 0) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            if self.register[x] == self.register[y] {
                self.pc += 2;
            }
        }

        // 0x6XNN: Sets VX to NN
        (6, _, _, _) => {
            let x = digit2 as usize;
            let nn = (opcode & 0xFF) as u8;
            self.register[x] = nn;
        }

        // 0x7XNN: Adds NN to VX (carry flag is not changed)
        (7, _, _, _) => {
            let x = digit2 as usize;
            let nn = (opcode & 0xFF) as u8;
            self.register[x] = self.register[x].wrapping_add(nn);
        }

        // 0x8XY0: Sets VX to the value of VY.
        (8, _, _, 0) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            self.register[x] = self.register[y];
        }

        // 0x8XY1: Sets VX to VX or VY. (bitwise OR operation)
        (8, _, _, 1) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            self.register[x] |= self.register[y];
        }

        // 0x8XY2: Sets VX to VX And VY. (bitwise AND operation)
        (8, _, _, 2) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            self.register[x] &= self.register[y];
        }

        // 0x8XY3: Sets VX to VX xor VY. (bitwise XOR operation)
        (8, _, _, 3) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            self.register[x] ^= self.register[y];
        }

        // 0x8XY4: adds VY to VX. VF is set to 1 when there's an overflow, and to 0 when there is not
        (8, _, _, 4) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            let (new_x, carry_flag) = self.register[x].overflowing_add(self.register[y]);
            self.register[x] = new_x;
            self.register[0xF] = if carry_flag {1} else {0}
        }

        // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's an underflow, and 1 when there is not
        (8, _, _, 5) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            let (new_x, borrow_flag) = self.register[x].overflowing_sub(self.register[y]);
            self.register[x] = new_x;
            self.register[0xF] = if borrow_flag {0} else {1}
        }

        // 0x8XY6: Shifts VX to the right by 1, then stores the least significant bit of VX prior to the shift into VF.
        (8, _, _, 6) => {
            let x = digit2 as usize;
            let lsb = 1 & self.register[x];
            self.register[x] >>= 1;
            self.register[0xF] = lsb;
        }

        // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's an underflow, and 1 when there is not
        (8, _, _, 7) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            let (new_x, borrow_flag) = self.register[y].overflowing_sub(self.register[x]);
            self.register[x] = new_x;
            self.register[0xF] = if borrow_flag {0} else {1}
        }

        // 0x8XYE: Shifts VX to the left by 1, then sets VF to 1 if the most significant bit of VX prior to that shift was set, or to 0 if it was unset
        (8, _, _, 0xE) => {
            let x = digit2 as usize;
            let msb = (self.register[x] >> 7) & 1;
            self.register[x] <<= 1;
            self.register[0xF] = msb;
        }

        // 0x9XY0: Skips the next instruction if VX does not equal VY
        (9, _, _, 0) => {
            let x = digit2 as usize;
            let y = digit3 as usize;
            if self.register[x] != self.register[y] {
                self.pc += 2;
            }
        }

        // 0xANNN: Sets I to the address NNN
        (0xA, _, _, _) => {
            let nnn = (opcode & 0xFFF) as u8;
            self.i_reg = nnn as u16;
        }

        // 0xBNNN: Jumps to the address NNN plus V0
        (0xB, _, _, _) => {
            let nnn = (opcode & 0xFFF) as u8;
            self.pc = (self.register[0] + nnn) as u16;
        }

        // 0xCXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN
        (0xC, _, _, _) => {
            let x = digit2 as usize;
            let nn = (opcode & 0xFF) as u8;
            let rng: u8 = random();
            self.register[x] = rng & nn;
        }

        // 0xDXYN: Draw a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels
        (0xD, _, _, _) => {
            let x_coord = self.register[digit2 as usize] as u16;
            let y_coord = self.register[digit3 as usize] as u16;
            let total_rows = digit4;
            // Keep track if any pixels were flipped
            let mut flipped = false;

            for y_line in 0..total_rows  {
                let addr = self.i_reg + y_line as u16;
                let pixels = self.memory[addr as usize];
                // Iterate over each column in our row
                for x_line in 0..8 {
                    // Use a mask to fetch current pixel's bit. Only flip if a 1
                    if (pixels & (0b1000_0000 >> x_line)) != 0 {
                        // Sprites should wrap around screen, so apply modulo
                        let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                        let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                        // Get our pixel's index for our 1D screen array
                        let idx = x + SCREEN_WIDTH * y;
                        // Check if we're about to flip the pixel and set
                        flipped |= self.display[idx];
                        self.display[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.register[0xF] = 1;
                } else {
                    self.register[0xF] = 0;
                }

            }


        // 0xEX9E: skips the next instruction if the key stored in VX(only consider the lowest nibble) is pressed
        (0xE, _, 9, 0xE) => {
            let x = digit2 as usize;
            let vx = self.register[x];
            let key = self.keys[vx as usize];
            if key {
                self.pc += 2;
            }
        }

        // 0xEXA1: Skips the next instruction if the key stored in VX(only consider the lowest nibble) is not pressed

        (0xE, _, 0xA, 1) => {
            let x = digit2 as usize;
            let vx = self.register[x];
            let key = self.keys[vx as usize];
            if !key {
                self.pc += 2;
            }
        }

        // 0xFX07: Sets VX to the value of the delay timer
        (0xF, _, 0, 7) => {
            let x = digit2 as usize;
            self.register[x] = self.delay_timer;
        }


        // 0xFX0A: A key press is awaited, and then stored in VX
        (0xF, _, 0, 0xA) => {
            let x = digit2 as usize;
            let mut pressed = false;
            for i in 0..self.keys.len() {
                if self.keys[i] {
                    self.register[x] = i as u8;
                    pressed = true;
                    break;
                }
            }
            if !pressed {
                // Redo opcode
                self.pc -= 2;
            }
        }

        // 0xFX15: Sets the delay timer to VX.
        (0xF, _, 1, 5) => {
            let x = digit2 as usize;
            self.delay_timer = self.register[x];

        }

        // 0xFX18: Sets the sound timer to VX
        (0xF, _, 1, 8) => {
            let x = digit2 as usize;
            self.sound_timer = self.register[x];

        }

        // 0xFX1E: Adds VX to I. VF is not affected
        (0xF, _, 1, 0xE) => {
            let x = digit2 as usize;
            let vx = self.register[x] as u16;
            self.i_reg = self.i_reg.wrapping_add(vx);
        }

        // 0xFX29: Sets I to the location of the sprite for the character in VX
        (0xF, _, 2, 9) => {
            let x = digit2 as usize;
            let character = self.register[x] as u16;
            self.i_reg = character * 5;       // Since all of our font sprites take up five bytes each, their RAM address is simply their value times 5.
        }

        // 0xFX33: Stores the binary-coded decimal representation of VX,
        //         with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2









        // Catch-all for unimplemented opcodes
        _ => {
            unimplemented!("Unimplemented opcode: 0x{:04X}", opcode);
        }
    }
    }
}
