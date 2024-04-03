pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80; //16 chars * 5 bytes each
const FONTSET: [u8; FONTSET_SIZE] = [ //all hexadecimal digits represented as sprites
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

const RAM_SIZE: usize = 4096;
const NUM_REG: usize = 16;
const NUM_KEYS: usize = 16; //16 key keyboard
const STACK_SIZE: usize = 16;

const START_ADDR: u16 = 0x200; //Program is loaded at offset of 0x200 in the RAM

use rand::random;

pub struct Emu {
    pc: u16, //program counter
    ram: [u8; RAM_SIZE], //array of bytes i.e. word size = 8

    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], //chip8 is monochrome i.e. only black and white colors
    v_reg: [u8; NUM_REG], //16 general purpose registers V0 -> VF
    i_reg: u16, //ma or memory address register
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    sp: u16, //stack pointer
    dt: u8, //delay timer
    st: u8, //sound timer
}

impl Emu {
    pub fn new() -> Emu {
        let mut e = Emu {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REG],
            i_reg: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            sp: 0,
            dt: 0,
            st: 0,
        };
        e.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        e
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REG];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let _opr = self.fetch(); //fetch
        //decode and execute

    }

    fn _execute(&mut self, op: u16) {
        //decode
        let dig1 = (op & 0xF000) >> 12;
        let dig2 = (op & 0x0F00) >> 8;
        let dig3 = (op & 0x00F0) >> 4;
        let dig4 = op & 0x000F;

        //execute
        match (dig1, dig2, dig3, dig4) {
            //0x0000 NOP
            (0, 0, 0, 0) => return,
            //0x00E0 CLS
            (0, 0, 0xE, 0) => self.screen.fill(false),
            //0x00EE Return from subroutine
            (0, 0, 0xE, 0xE) => {
                let addr = self.pop();
                self.pc = addr;
            }
            //0x1NNN JMP NNN
            (1, _, _, _) => {
                let addr = 0xFFF & op; //or dig1 << 12 | dig2 << 8 | dig3 << 4 } dig4
                self.pc = addr;
            }
            //0x2NNN Call subroutine
            (2, _, _, _) => {
                self.push(self.pc);
                let addr = 0xFFF & op;
                self.pc = addr;
            }
            //0x3XNN skip next if VX = NN
            (3, _, _, _) => {
                let x = 0xF00 & op;
                if self.v_reg[x as usize] == 0xFF & op as u8 {
                    self.pc += 2;
                }
            }
            //0x4XNN skip next if VX !+ NN
            (4, _, _, _) => {
                let x = 0xF00 & op;
                if self.v_reg[x as usize] != 0xFF & op as u8 {
                    self.pc += 2;
                }
            }
            //0x5XY0 skip next if VX == VY
            (5, _, _, 0) => {
                let x = 0xF00 & op;
                let y = 0xF0 & op;
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            //0x6XNN set VX = NN
            (6, _, _, _) => {
                let x = 0xF00 & op;
                let nn = 0xFF & op as u8;
                self.v_reg[x as usize] = nn;
            }
            //0x7XNn VX += NN
            //carry flag isnt affected by this operation
            (7, _, _, _) => {
                let x = 0xF00 & op as usize;
                let nn = 0xFF & op as u8;
                //overflow will cause panic
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            //0x8XY0 VX = VY
            (8, _, _, 0) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            //0x8XY1 VX |= VY
            (8, _, _, 1) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            //0x8VY2 VX &= VY
            (8, _, _, 2) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            //0x8VY3 VX ^= VY
            (8, _, _, 3) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }
            //0x8VY4 VX += VY with carry flag
            //if carry VF = 1 since VF is also the flag register
            (8, _, _, 4) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                let (res, over) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                if over {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
                self.v_reg[x] = res;
            }
            //0x8XY5 VX >> 1, store dropped bit in VF
            (8, _, _, 5) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                let (res, und) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                if und {
                    self.v_reg[0xF] = 0;
                } else {
                    self.v_reg[0xF] = 1;
                }
                self.v_reg[x] = res;

            }
            //0x8XY6 VX >> 1, store dropped bit in VF
            (8, _, _, 6) => {
                let x = 0xF00 & op as usize;
                self.v_reg[0xF] = self.v_reg[x] & 1; //dropped bit
                self.v_reg[x] >>= 1;

            }
            //0x8XY7 VY -= VX same as 0x8XY5 but opposite direction
            (8, _, _, 7) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                let (res, und) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                if und {
                    self.v_reg[0xF] = 0;
                } else {
                    self.v_reg[0xF] = 1;
                }
                self.v_reg[x] = res;

            }
            //0x8XYE VX << 1, overflowed bit in VF
            (8, _, _, 0xE) => {
                let x = 0xF00 & op as usize;
                self.v_reg[0xF] = self.v_reg[x] & 0x80; //dropped bit
                self.v_reg[x] <<= 1;
            }
            //0x9XY0 skip if VX != VY
            (9, _, _, 0) => {
                let x = 0xF00 & op as usize;
                let y = 0xF0 & op as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            //0xANNN set I <- NNN
            (0xA, _, _, _) => {
                self.i_reg = 0xFFF & op;
            }
            //0xBNNN JMP V0 + 0xNNN
            (0xB, _, _, _) => {
                self.pc = self.v_reg[0] as u16 + op & 0xFFF;
            }
            //0xCXNN VX = rand() & NN
            (0xC, _, _, _) => {
                let x = 0xF00 & op as usize;
                let nn = 0xFF & op as u8;
                let r: u8 = random();

                self.v_reg[x] = r & nn;
            }
            //0xDXYN draw sprites, get (x,y) from VX, VY, N is number of rows
            (0xD, _, _, _) => {
                //xy coords
                let xcoord = 0xF00 & op as usize;
                let ycoord = 0xF0 & op as usize;
                //num rows = height of sprite
                let n = 0xF & op as usize;
                
                let mut flipped = false;
                for line in 0..n {
                    let addr = self.i_reg + line as u16;
                    //base pixel address
                    let pixels = self.ram[addr as usize];

                    for col in 0..8 {
                        //basically get leftmost bit value to rightmost, i.e. first to lsat column
                        if pixels & (0b1000_0000 >> col) != 0 {
                            let x = (xcoord + col) as usize % SCREEN_WIDTH;
                            let y = (ycoord + line) as usize % SCREEN_HEIGHT;

                            //index of bit
                            let idx = x + y * SCREEN_WIDTH; //same concept as 2d indexing
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }
            //0xEX9E skip if keys[VX] is pressed
            (0xE, _, 9, 0xE) => {
                let x = 0xF00 & op as usize;
                if self.keys[self.v_reg[x] as usize] {
                    self.pc += 2
                }
            }
            //0xEXA1 same as above but skip if not pressed
            (0xE, _, 0xA, 0x1) => {
                let x = 0xF00 & op as usize;
                if !self.keys[self.v_reg[x] as usize] {
                    self.pc += 2;
                }
            }
            //0xFX07 VX = DT
            (0xF, _, 0x0, 0x7) => {
                let x = 0xF00 & op as usize;
                self.v_reg[x] = self.dt;
            }
            //waits undlessly till key pressed, store it in VX
            //ticks continue while waiting
            //0xFX0A wait for key press - blocking
            (0xF, _, 0x0, 0xA) => {
                let x = 0xF00 & op as usize;
                let mut pressed = false;
                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            //0xFX15 DT = VX
            (0xF, _, 0x1, 0x5) => {
                let x = 0xF00 & op as usize;
                self.dt = self.v_reg[x];
            }
            //0xFX18 ST = VX
            (0xF, _, 0x1, 0x8) => {
                let x = 0xF00 & op as usize;
                self.st = self.v_reg[x];
            }
            //0xFX1E I += VX
            (0xF, _, 0x1, 0xE) => {
                let x = 0xF00 & op as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            }
            //0xFX29 Set I to font character in VX.
            (0xF, _, 0x2, 0x9) => {
                let x = 0xF00 & op as usize;
                //each char in FONTSET occupies 5 bytes
                //since they are stored starting from 0
                //ith sprite address = 5 * i
                self.i_reg = 5 * self.v_reg[x] as u16;
            }
            //0xFX33 store BCD encoding of VX in I
            (0xF, _, 0x3, 0x3) => {
                let x = 0xF00 & op as usize;
                //since VX is u8, BCD would be off max 3 bytes as max value of u8 is 255 or 0xFF
                //implement using double babble later on
                let vx = self.v_reg[x];
                let hun = vx / 100;
                let ten = (vx / 10) % 10;
                let ones = vx % 10;

                self.ram[self.i_reg as usize] = hun;
                self.ram[self.i_reg as usize + 1] = ten;
                self.ram[self.i_reg as usize + 2] = ones;
            }
            //0xFX55 store V0 -> VX inclusive in memory starting from address at I
            (0xF, _, 0x5, 0x5) => {
                let x = 0xF00 & op as usize;
                for i in 0..=x {
                    self.ram[self.i_reg as usize + i] = self.v_reg[i as usize];
                }
            }
            //0xFX65 reverse of above, store into V0 -> VX
            (0xF, _, 0x6, 0x5) => {
                let x = 0xF00 & op as usize;
                for i in 0..=x {
                    self.v_reg[i as usize] = self.ram[self.i_reg as usize + i];
                }
            }
            (_, _, _, _) => panic!("invalid sequence of bytes => {}", op),
        }

    }

    fn fetch(&mut self) -> u16 { //each instruction is upto 2 bytes long
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[self.pc as usize + 1] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }
    
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                //beep logic
            }
            self.st -= 1;
        }
    }
}