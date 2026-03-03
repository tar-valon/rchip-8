//! A small and straightforward CHIP-8 emulator core.
//!
//! This module contains the full CPU state, memory, timers,
//! input handling, and opcode execution logic.
//!
//! It does not deal with rendering or audio directly —
//! it only exposes state that an external frontend can use.

use rand::random;

/// CHIP-8 display width (64 pixels)
pub const SCREEN_WIDTH: usize = 64;

/// CHIP-8 display height (32 pixels)
pub const SCREEN_HEIGHT: usize = 32;

/// Total number of pixels in the display buffer
pub const TOTAL_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Total memory size (4KB)
pub const MEMORY_SIZE: usize = 4096;

/// Maximum stack depth (CHIP-8 supports 16 nested calls)
pub const STACK_SIZE: usize = 16;

/// Number of general purpose registers (V0–VF)
pub const TOTAL_REGISTER: usize = 16;

/// Total number of keys (hex keypad: 0x0–0xF)
pub const TOTAL_KEYS: usize = 16;

/// Programs start at memory address 0x200
const START_ADDR: u16 = 0x200;

/// Size of the built-in fontset (16 chars × 5 bytes each)
pub const FONTSET_SIZE: usize = 80;

/// Built-in fontset stored at the beginning of memory.
/// Each character is 5 bytes tall and represents digits 0–F.
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

/// Represents the full state of the CHIP-8 virtual machine.
pub struct Emulator {
    /// Program counter – points to the current instruction.
    pc: u16,

    /// 4KB memory space.
    memory: [u8; MEMORY_SIZE],

    /// General-purpose registers V0–VF.
    /// VF is also used as a flag register.
    register: [u8; TOTAL_REGISTER],

    /// Index register (I) – usually used for memory addresses.
    i_reg: u16,

    /// Call stack for subroutines.
    stack: Vec<u16>,

    /// Monochrome display buffer.
    /// `true` means pixel is on.
    display: [bool; TOTAL_PIXELS],

    /// Delay timer (counts down at 60Hz).
    delay_timer: u8,

    /// Sound timer (also counts down at 60Hz).
    sound_timer: u8,

    /// Current key states.
    keys: [bool; TOTAL_KEYS],
}

impl Emulator {
    /// Creates a new emulator instance with clean state
    /// and loads the built-in fontset into memory.
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

        // Load fontset into the beginning of memory
        new_emu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    /// Resets the entire emulator state.
    /// Useful when loading a new ROM.
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

        // Reload the fontset after clearing memory
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    /// Fetches the next 2-byte opcode from memory
    /// and advances the program counter.
    fn fetch(&mut self) -> u16 {
        let msb = self.memory[self.pc as usize] as u16;
        let lsb = self.memory[(self.pc + 1) as usize] as u16;

        self.pc += 2; // Move to next instruction

        (msb << 8) | lsb
    }

    /// Executes one full CPU cycle:
    /// fetch → decode → execute.
    pub fn cpu_cycle(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    /// Decrements delay and sound timers.
    /// Should be called at 60Hz.
    pub fn tick_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: trigger a beep here
            }
            self.sound_timer -= 1;
        }
    }

    /// Returns a reference to the display buffer.
    /// A renderer can use this to draw the screen.
    pub fn get_display(&self) -> &[bool] {
        &self.display
    }

    /// Updates the state of a key.
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    /// Loads a ROM into memory starting at 0x200.
    pub fn load_game(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    /// Decodes and executes a single opcode.
    /// The opcode is split into 4 nibbles to make matching easier.
    fn execute(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // 0000 - No-op (ignored)
            (0, 0, 0, 0) => return,

            // 00E0 - Clear the display
            (0, 0, 0xE, 0) => self.display = [false; TOTAL_PIXELS],

            // 00EE - Return from subroutine
            (0, 0, 0xE, 0xE) => {
                if let Some(ret) = self.stack.pop() {
                    self.pc = ret;
                } else {
                    eprintln!("Stack underflow!");
                }
            }

            // 1NNN - Jump to address NNN
            (1, _, _, _) => self.pc = opcode & 0x0FFF,

            // 2NNN - Call subroutine at NNN
            (2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = opcode & 0x0FFF;
            }

            // 3XNN - Skip next instruction if VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.register[x] == nn {
                    self.pc += 2;
                }
            }

            // 4XNN - Skip if VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.register[x] != nn {
                    self.pc += 2;
                }
            }

            // 5XY0 - Skip if VX == VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.register[x] == self.register[y] {
                    self.pc += 2;
                }
            }

            // 6XNN - Set VX = NN
            (6, _, _, _) => self.register[digit2 as usize] = (opcode & 0xFF) as u8,

            // 7XNN - Add NN to VX (no carry flag affected)
            (7, _, _, _) => {
                let x = digit2 as usize;
                self.register[x] = self.register[x].wrapping_add((opcode & 0xFF) as u8);
            }

            // 8XY0 - VX = VY
            (8, _, _, 0) => self.register[digit2 as usize] = self.register[digit3 as usize],

            // 8XY1 - VX |= VY
            (8, _, _, 1) => self.register[digit2 as usize] |= self.register[digit3 as usize],

            // 8XY2 - VX &= VY
            (8, _, _, 2) => self.register[digit2 as usize] &= self.register[digit3 as usize],

            // 8XY3 - VX ^= VY
            (8, _, _, 3) => self.register[digit2 as usize] ^= self.register[digit3 as usize],

            // 8XY4 - Add VY to VX, set VF on carry
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (val, carry) = self.register[x].overflowing_add(self.register[y]);
                self.register[x] = val;
                self.register[0xF] = if carry { 1 } else { 0 };
            }

            // 8XY5 - VX -= VY, set VF to NOT borrow
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (val, borrow) = self.register[x].overflowing_sub(self.register[y]);
                self.register[x] = val;
                self.register[0xF] = if borrow { 0 } else { 1 };
            }

            // 8XY6 - Shift VX right by 1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                self.register[0xF] = self.register[x] & 1;
                self.register[x] >>= 1;
            }

            // 8XY7 - VX = VY - VX
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (val, borrow) = self.register[y].overflowing_sub(self.register[x]);
                self.register[x] = val;
                self.register[0xF] = if borrow { 0 } else { 1 };
            }

            // 8XYE - Shift VX left by 1
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                self.register[0xF] = (self.register[x] >> 7) & 1;
                self.register[x] <<= 1;
            }

            // 9XY0 - Skip if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.register[x] != self.register[y] {
                    self.pc += 2;
                }
            }

            // ANNN - Set I = NNN
            (0xA, _, _, _) => self.i_reg = opcode & 0x0FFF,

            // BNNN - Jump to NNN + V0
            (0xB, _, _, _) => self.pc = self.register[0] as u16 + (opcode & 0x0FFF),

            // CXNN - VX = random byte AND NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.register[x] = random::<u8>() & nn;
            }

            // DXYN - Draw sprite at (VX, VY)
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

                // VF is set if any pixels were flipped from on → off
                self.register[0xF] = if flipped { 1 } else { 0 };
            }

            // EX9E - Skip if key in VX is pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                if self.keys[self.register[x] as usize] {
                    self.pc += 2;
                }
            }

            // EXA1 - Skip if key in VX is NOT pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                if !self.keys[self.register[x] as usize] {
                    self.pc += 2;
                }
            }

            // FX07 - VX = delay_timer
            (0xF, _, 0, 7) => self.register[digit2 as usize] = self.delay_timer,

            // FX0A - Wait for key press
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

                // If no key is pressed, repeat this instruction
                if !pressed {
                    self.pc -= 2;
                }
            }

            // FX15 - delay_timer = VX
            (0xF, _, 1, 5) => self.delay_timer = self.register[digit2 as usize],

            // FX18 - sound_timer = VX
            (0xF, _, 1, 8) => self.sound_timer = self.register[digit2 as usize],

            // FX1E - I += VX
            (0xF, _, 1, 0xE) => {
                self.i_reg = self
                    .i_reg
                    .wrapping_add(self.register[digit2 as usize] as u16)
            }

            // FX29 - Set I to sprite location for digit in VX
            (0xF, _, 2, 9) => self.i_reg = (self.register[digit2 as usize] as u16) * 5,

            // FX33 - Store BCD representation of VX at I, I+1, I+2
            (0xF, _, 3, 3) => {
                let i = self.i_reg as usize;
                let vx = self.register[digit2 as usize];
                self.memory[i] = vx / 100;
                self.memory[i + 1] = (vx / 10) % 10;
                self.memory[i + 2] = vx % 10;
            }

            // FX55 - Store V0..VX in memory starting at I
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for offset in 0..=x {
                    self.memory[i + offset] = self.register[offset];
                }
            }

            // FX65 - Read V0..VX from memory starting at I
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
