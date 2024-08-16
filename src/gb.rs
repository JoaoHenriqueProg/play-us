use crate::emulator::Emulator;

struct Cpu {
    bus_8: u8,
    bus_16: u16,
    memory: [u8; 1024 * 64],
    regs: [u8; 8],
    // AF, BC, DE, HL, by the gods what does it mean why this order
    pc: usize,
    ime: bool,
    sp: u16, // stack pointer
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            bus_8: 0,
            bus_16: 0,
            memory: [0; 1024 * 64],
            regs: [
                0x01, // A
                0xB0, // F
                0x00, // B
                0x13, // C
                0x00, // D
                0xD8, // E
                0x01, // H
                0x4D, // L
            ],
            pc: 0x100,
            ime: false,
            sp: 0xFFFE,
        }
    }
}

pub struct GameBoyEmulator {
    cpu: Cpu,
}

type Regs = usize;
const RegA: usize = 0;
const RegF: usize = 1;
const RegB: usize = 2;
const RegC: usize = 3;
const RegD: usize = 4;
const RegE: usize = 5;
const RegH: usize = 6;
const RegL: usize = 7;

impl GameBoyEmulator {
    pub fn new() -> GameBoyEmulator {
        GameBoyEmulator { cpu: Cpu::new() }
    }

    fn set_hram(&mut self, address: usize, value: u8) {
        self.cpu.memory[address + 0xFF00] = value;
        println!("Set {value} at {address} (or {}) of hram", address + 0xFF00);
    }
    fn get_hram(&mut self, address: usize) -> u8 {
        println!(
            "Got {} at {address} (or {}) of hram",
            self.cpu.memory[address + 0xFF00],
            address + 0xFF00
        );
        return self.cpu.memory[address + 0xFF00];
    }

    #[inline]
    fn set_n_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] | 1 << 6;
            }
            false => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] & !(1 << 6);
            }
        }
    }

    #[inline]
    fn set_z_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] | 1 << 7;
            }
            false => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] & !(1 << 7);
            }
        }
    }
    #[inline]
    fn get_z_flag(&mut self) -> bool {
        self.cpu.regs[RegF] & (1 << 7) != 0
    }

    #[inline]
    fn set_h_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] | 1 << 5;
            }
            false => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] & !(1 << 5);
            }
        }
    }

    #[inline]
    fn set_c_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] | 1 << 4;
            }
            false => {
                self.cpu.regs[RegF] = self.cpu.regs[RegF] & !(1 << 4);
            }
        }
    }

    #[inline]
    fn reg_inc_a(&mut self, to_inc: u8) {
        self.cpu.regs[RegA] = self.cpu.regs[RegA].wrapping_add(to_inc);
    }
    #[inline]
    fn reg_inc_l(&mut self, to_inc: u8) {
        self.cpu.regs[RegL] = self.cpu.regs[RegL].wrapping_add(to_inc);
    }

    #[inline]
    fn get_hl(&self) -> u16 {
        ((self.cpu.regs[RegH] as u16) << 8) | self.cpu.regs[RegL] as u16
    }

    #[inline]
    fn sub_reg_from_a(&mut self, reg: Regs) {
        let half_a: u8 = self.cpu.regs[RegA] & 0x0F;
        let half_r = self.cpu.regs[reg.clone()] & 0x0F;
        self.set_h_flag(half_a < half_r);

        self.set_c_flag(self.cpu.regs[reg] > self.cpu.regs[RegA]);

        self.cpu.regs[RegA] = self.cpu.regs[RegA].wrapping_sub(self.cpu.regs[reg]);

        self.set_z_flag(self.cpu.regs[RegA] == 0);
        self.set_n_flag(true);
    }

    fn print_regs(&self) {
        println!(
            "A={:02X} F={:02X}",
            self.cpu.regs[RegA], self.cpu.regs[RegF]
        );
        println!(
            "B={:02X} C={:02X}",
            self.cpu.regs[RegB], self.cpu.regs[RegC]
        );
        println!(
            "D={:02X} E={:02X}",
            self.cpu.regs[RegD], self.cpu.regs[RegE]
        );
        println!(
            "H={:02X} L={:02X}",
            self.cpu.regs[RegH], self.cpu.regs[RegL]
        );
    }

    fn op_dec_reg(&mut self, reg: Regs) {
        let half_reg = self.cpu.regs[reg] & 0x0F;
        let half_one = 1;
        self.set_h_flag(half_reg < half_one);

        self.cpu.regs[reg] = self.cpu.regs[reg].wrapping_sub(1);

        self.set_z_flag(self.cpu.regs[reg] == 0);
        self.set_n_flag(true);
    }
    fn op_inc_reg(&mut self, reg: Regs) {
        let half_reg = self.cpu.regs[reg] & 0x0F;
        let half_inc = 1;

        self.cpu.regs[reg] = self.cpu.regs[reg].wrapping_add(1);

        self.set_n_flag(false);
        self.set_z_flag(self.cpu.regs[reg] == 0);
        self.set_h_flag(half_inc + half_reg > 0xF);
    }

    // I know I probably shouldn't start directly implement opcodes, but preguicinha of doing
    // the game boy architecture and stuff
    fn compute(&mut self, rom: &[u8]) -> u64 {
        // returns the cpu cycles it takes, so in the future I can implement real cpu bottleneck
        print!("{:04X}: ", self.cpu.pc);
        let checked_functions: [u8; 15] = [
            0, 0xC3, 0xAF, 0x21, 0x0E, 0x06, 0x32, 0x05, 0x20, 0x0D, 0x3E, 0xF3, 0xE0, 0xF0, 0xFE,
        ];
        if !checked_functions.contains(&self.cpu.memory[self.cpu.pc]) {
            // panic!("{:02X}", self.cpu.memory[self.cpu.pc])
        }
        match self.cpu.memory[self.cpu.pc] {
            // 5B (LD E,E)
            0 | 0x5B => {
                println!("NOP");
                self.cpu.pc += 1;
                return 4;
            }
            0x05 => {
                // check for half carry
                println!("DEC B");
                self.op_dec_reg(RegB);
                self.cpu.pc += 1;
                return 4;
            }
            0x06 => {
                println!("LD B,d8");
                self.cpu.regs[RegB] = self.cpu.memory[self.cpu.pc + 1];
                self.cpu.pc += 2;
                return 8;
            }
            0x0D => {
                println!("INC C");
                self.op_inc_reg(RegC);
                self.cpu.pc += 1;
                return 4;
            }
            0x0D => {
                println!("DEC C");
                self.op_dec_reg(RegC);
                self.cpu.pc += 1;
                return 4;
            }
            0x0E => {
                println!("LD C,d8");
                self.cpu.regs[RegC] = self.cpu.memory[self.cpu.pc + 1];
                self.cpu.pc += 2;
                return 8;
            }
            0x11 => {
                println!("LD DE,d16");
                self.cpu.regs[RegD] = self.cpu.memory[self.cpu.pc + 1];
                self.cpu.regs[RegE] = self.cpu.memory[self.cpu.pc + 2];
                self.cpu.pc += 3;
                return 12;
            }
            0x20 => {
                println!("JR NZ,r8");
                // com branch: 12
                // sem branch: 8
                if self.get_z_flag() {
                    self.cpu.pc += 2;
                    return 8;
                }
                self.cpu.pc += 1;
                // pura gambiarra
                // todo: maybe fix this later
                self.cpu.pc = (self.cpu.pc as i128 + (self.cpu.memory[self.cpu.pc] as i8) as i128)
                    as usize
                    + 1;
                return 12;
            }
            0x21 => {
                println!("LD HL,d16");
                // regs -> h l -- mem -> x y
                //         y x
                self.cpu.regs[RegL] = self.cpu.memory[self.cpu.pc + 1];
                self.cpu.regs[RegH] = self.cpu.memory[self.cpu.pc + 2];
                self.cpu.pc += 3;
                return 12;
            }
            0x2A => {
                println!("LD A,(HL+)");
                self.cpu.regs[RegA] = self.cpu.memory[self.get_hl() as usize]; // SINCE WHEN WAS THIS LINE COMMENTED
                self.cpu.regs[RegL] = self.cpu.regs[RegL].wrapping_add(1);

                if self.cpu.regs[RegL] == 0 {
                    // number wraped around
                    self.cpu.regs[RegH] = self.cpu.regs[RegH].wrapping_add(1);
                }
                self.cpu.pc += 1;
                return 8;
            }
            0x2C => {
                println!("INC L");
                // check for half carry first of all
                self.op_inc_reg(RegL);
                self.cpu.pc += 1;
                return 4;
            }
            0x31 => {
                println!("LD SP,d16");
                let mut value_d16 = 0;
                value_d16 |= self.cpu.memory[self.cpu.pc + 1] as usize;
                value_d16 |= (self.cpu.memory[self.cpu.pc + 2] as usize) << 8;
                self.cpu.sp = value_d16 as u16;
                self.cpu.pc += 3;
                return 12;
            }
            0x32 => {
                println!("LD (HL-),A");
                let mut address = 0;
                address |= ((self.get_hl() & 0x0F) as usize) << 8; // putting l left
                address |= ((self.get_hl() & 0xF0) as usize) >> 8; // putting h right
                                                                   // by the gods please be it ? I'm not sure
                self.cpu.memory[address as usize] = self.cpu.regs[RegA]; // SINCE WHEN WAS THIS LINE COMMENTED
                self.cpu.regs[RegL] = self.cpu.regs[RegL].wrapping_sub(1);

                if self.cpu.regs[RegL] == 255 {
                    // number wraped around
                    self.cpu.regs[RegH] = self.cpu.regs[RegH].wrapping_sub(1);
                }
                self.cpu.pc += 1;
                return 8;
            }
            0x36 => {
                println!("LD (HL),d8");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.memory[self.cpu.pc + 1];
                self.cpu.pc += 2;
                return 12;
            }
            0x3C => {
                println!("INC A");
                self.op_inc_reg(RegA);
                self.cpu.pc += 1;
                return 4;
            }
            0x3E => {
                println!("LD A,d8");
                self.cpu.regs[RegA] = self.cpu.memory[self.cpu.pc + 1];
                self.cpu.pc += 2;
                return 8;
            }
            0x4A => {
                println!("LD C,D");
                self.cpu.regs[RegC] = self.cpu.regs[RegD];
                self.cpu.pc += 1;
                return 4;
            }
            0x4B => {
                println!("LD C,E");
                self.cpu.regs[RegC] = self.cpu.regs[RegE];
                self.cpu.pc += 1;
                return 4;
            }
            0x53 => {
                println!("LD D,E");
                self.cpu.regs[RegD] = self.cpu.regs[RegE];
                self.cpu.pc += 1;
                return 4;
            }
            0x55 => {
                println!("LD D,L");
                self.cpu.regs[RegD] = self.cpu.regs[RegL];
                self.cpu.pc += 1;
                return 4;
            }
            0x56 => {
                println!("LD D,(HL)");
                self.cpu.regs[RegD] = self.cpu.memory[self.get_hl() as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x57 => {
                println!("LD D,A");
                self.cpu.regs[RegD] = self.cpu.regs[RegA];
                self.cpu.pc += 1;
                return 4;
            }
            0x58 => {
                println!("LD E,B");
                self.cpu.regs[RegE] = self.cpu.regs[RegB];
                self.cpu.pc += 1;
                return 4;
            }
            0x59 => {
                println!("LD E,C");
                self.cpu.regs[RegE] = self.cpu.regs[RegC];
                self.cpu.pc += 1;
                return 4;
            }
            0x5A => {
                println!("LD E,D");
                self.cpu.regs[RegE] = self.cpu.regs[RegD];
                self.cpu.pc += 1;
                return 4;
            }
            0x5C => {
                println!("LD E,H");
                self.cpu.regs[RegE] = self.cpu.regs[RegH];
                self.cpu.pc += 1;
                return 4;
            }
            0x6C => {
                println!("LD L,H");
                self.cpu.regs[RegL] = self.cpu.regs[RegH];
                self.cpu.pc += 1;
                return 4;
            }
            0x6E => {
                println!("LD L,(HL)");
                self.cpu.regs[RegL] = self.cpu.memory[self.get_hl() as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x6F => {
                println!("LD L,A");
                self.cpu.regs[RegL] = self.cpu.regs[RegA];
                self.cpu.pc += 1;
                return 4;
            }
            0x71 => {
                println!("LD (HL),C");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegC];
                self.cpu.pc += 1;
                return 8;
            }
            0x72 => {
                println!("LD (HL),D");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegD];
                self.cpu.pc += 1;
                return 8;
            }
            0x73 => {
                println!("LD (HL),E");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegE];
                self.cpu.pc += 1;
                return 8;
            }
            0x74 => {
                println!("LD (HL),H");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegH];
                self.cpu.pc += 1;
                return 8;
            }
            0x75 => {
                println!("LD (HL),L");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegL];
                self.cpu.pc += 1;
                return 8;
            }
            0x90 => {
                println!("SUB B");
                self.sub_reg_from_a(RegB);
                self.cpu.pc += 1;
                return 4;
            }
            0x91 => {
                println!("SUB C");
                self.sub_reg_from_a(RegC);
                self.cpu.pc += 1;
                return 4;
            }
            0x92 => {
                println!("SUB D");
                self.sub_reg_from_a(RegD);
                self.cpu.pc += 1;
                return 4;
            }
            0x93 => {
                println!("SUB E");
                self.sub_reg_from_a(RegE);
                self.cpu.pc += 1;
                return 4;
            }
            0x94 => {
                println!("SUB H");
                self.sub_reg_from_a(RegH);
                self.cpu.pc += 1;
                return 4;
            }
            0x95 => {
                println!("SUB L");
                self.sub_reg_from_a(RegL);
                self.cpu.pc += 1;
                return 4;
            }
            0xAF => {
                println!("XOR A");
                self.cpu.regs[RegA] = 0;
                self.set_z_flag(true);
                self.set_c_flag(false);
                self.set_h_flag(false);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                return 4;
            }
            0xC3 => {
                println!("JMP a16");
                // and this is where I learnt the difference between big and small endian
                // now I just wonder where else have I not flipped the bytes where I should
                let mut new_address = 0;
                new_address |= self.cpu.memory[self.cpu.pc + 1] as usize;
                new_address |= (self.cpu.memory[self.cpu.pc + 2] as usize) << 8;
                self.cpu.pc = new_address as usize;
                return 16;
            }
            0xEA => {
                println!("LD (a16),A");
                let mut address = 0;
                address |= self.cpu.memory[self.cpu.pc + 1] as usize;
                address |= (self.cpu.memory[self.cpu.pc + 2] as usize) << 8;
                self.cpu.memory[address as usize] = self.cpu.regs[RegA];
                self.cpu.pc += 3;
                return 16;
            }
            0xE0 => {
                println!("LDH (a8),A");
                self.set_hram(
                    self.cpu.memory[self.cpu.pc + 1] as usize,
                    self.cpu.regs[RegA],
                );
                self.cpu.pc += 2;
                return 12;
            }
            0xF0 => {
                println!("LDH A,(a8)");
                self.cpu.regs[RegA] = self.get_hram(self.cpu.memory[self.cpu.pc + 1] as usize);
                // I spent almost a whole day trying to find out why where the game in an infinite loop
                // Till I had the great idea of using the concept of "searching online"
                // Turns out the game keeps waiting for the game to draw, which is when 0xFF44 (the y lcd counter)
                // Is 148 (or whatever it is in hex)  
                self.cpu.pc += 2;
                return 12;
            }
            0xF3 => {
                println!("DI");
                self.cpu.ime = false;
                self.cpu.pc += 1;
                return 4;
            }
            0xFE => {
                println!("CP d8");
                let value = self.cpu.memory[self.cpu.pc + 1];
                let half_a: u8 = self.cpu.regs[RegA] & 0x0F;
                let half_v = value & 0x0F;
                self.set_h_flag(half_a < half_v);

                self.set_c_flag(value > self.cpu.regs[RegA]);

                let result = self.cpu.regs[RegA].wrapping_sub(value);
                self.set_z_flag(result == 0);
                self.set_n_flag(true);
                self.cpu.pc += 2;
                return 8;
            }
            _ => {
                println!();
                self.print_regs();
                todo!(
                    "{:02X}\nPC: {:04X} | {}",
                    self.cpu.memory[self.cpu.pc],
                    self.cpu.pc,
                    self.cpu.pc
                )
            }
        }
    }
}

impl Emulator for GameBoyEmulator {
    fn run(&mut self, rom: &[u8]) {
        // for now it doesn't matter, the tetris rom doesn't have bank switching (as far as I now)
        if rom.len() > 1024 * 32 {
            todo!("rom too big, implement bank switching")
        }

        self.cpu.memory[0..rom.len()].copy_from_slice(rom);

        // let mut steps = 4;
        loop {
            // if steps == 0 {
            //     self.print_regs();
            //     break;
            // }
            self.compute(rom);
            // if self.cpu.pc == 0x100 {
            //     self.print_regs();
            //     break;
            // }
            // steps -= 1;
        }
    }
}
