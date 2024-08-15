use crate::emulator::Emulator;

struct Cpu {
    bus_8: u8,
    bus_16: u16,
    memory: [u8; 1024 * 64],
    regs: [u8; 8],
    // AF, BC, DE, HL, by the gods what does it mean why this order
    pc: usize,
    ime: bool,
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            bus_8: 0,
            bus_16: 0,
            memory: [0; 1024 * 64],
            regs: [0; 8],
            pc: 0x100,
            ime: false
        }
    }
}

pub struct GameBoyEmulator {
    cpu: Cpu,
}

#[derive(Clone, Copy)]
enum Regs {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl GameBoyEmulator {
    pub fn new() -> GameBoyEmulator {
        GameBoyEmulator { cpu: Cpu::new() }
    }

    #[inline]
    fn set_n_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] | 1 << 6;
            }
            false => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] & !(1 << 6);
            }
        }
    }

    #[inline]
    fn set_z_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] | 1 << 7;
            }
            false => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] & !(1 << 7);
            }
        }
    }
    #[inline]
    fn get_z_flag(&mut self) -> bool {
        self.cpu.regs[Regs::F as usize] & (1 << 7) != 0
    }

    #[inline]
    fn set_h_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] | 1 << 5;
            }
            false => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] & !(1 << 5);
            }
        }
    }

    #[inline]
    fn set_c_flag(&mut self, state: bool) {
        match state {
            true => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] | 1 << 4;
            }
            false => {
                self.cpu.regs[Regs::F as usize] = self.cpu.regs[Regs::F as usize] & !(1 << 4);
            }
        }
    }

    #[inline]
    fn reg_inc_a(&mut self, to_inc: u8) {
        self.cpu.regs[Regs::A as usize] = self.cpu.regs[Regs::A as usize].wrapping_add(to_inc);
    }
    #[inline]
    fn reg_dec_b(&mut self, to_dec: u8) {
        self.cpu.regs[Regs::B as usize] = self.cpu.regs[Regs::B as usize].wrapping_sub(to_dec);
    }
    fn reg_dec_c(&mut self, to_dec: u8) {
        self.cpu.regs[Regs::B as usize] = self.cpu.regs[Regs::C as usize].wrapping_sub(to_dec);
    }
    #[inline]
    fn reg_inc_l(&mut self, to_inc: u8) {
        self.cpu.regs[Regs::L as usize] = self.cpu.regs[Regs::L as usize].wrapping_add(to_inc);
    }

    #[inline]
    fn get_hl(&self) -> u16 {
        ((self.cpu.regs[Regs::H as usize] as u16) << 8) | self.cpu.regs[Regs::L as usize] as u16
    }

    #[inline]
    fn sub_reg_from_a(&mut self, reg: Regs) {
        let half_a: u8 = self.cpu.regs[Regs::A as usize] & 0x0F;
        let half_r = self.cpu.regs[reg.clone() as usize] & 0x0F;
        self.set_h_flag(half_a < half_r);

        self.set_c_flag(self.cpu.regs[reg as usize] > self.cpu.regs[Regs::A as usize]);

        self.cpu.regs[Regs::A as usize] =
            self.cpu.regs[Regs::A as usize].wrapping_sub(self.cpu.regs[reg as usize]);

        self.set_z_flag(self.cpu.regs[Regs::A as usize] == 0);
        self.set_n_flag(true);
    }

    // I know I probably shouldn't start directly implement opcodes, but preguicinha of doing
    // the game boy architecture and stuff
    fn compute(&mut self, rom: &[u8]) -> u64 {
        // returns the cpu cycles it takes, so in the future I can implement real cpu bottleneck
        print!("{:04X}: ", self.cpu.pc);
        match rom[self.cpu.pc] {
            // 5B (LD E,E)
            0 | 0x5B => {
                println!("NOP");
                self.cpu.pc += 1;
                return 4;
            }
            0x05 => {
                // check for half carry
                println!("DEC B");
                let half_b = self.cpu.regs[Regs::B as usize] & 0x0F;
                let half_one = 1 & 0x0F;
                self.set_h_flag(half_b < half_one);

                self.reg_dec_b(1);

                self.set_z_flag(self.cpu.regs[Regs::B as usize] == 0);
                self.set_n_flag(true);
                self.cpu.pc += 1;
                return 4;
            }
            0x06 => {
                println!("LD B,d8");
                self.cpu.pc += 1;
                self.cpu.regs[Regs::B as usize] = self.cpu.memory[self.cpu.pc];
                self.cpu.pc += 1;
                return 8;
            }
            0x0D => {
                println!("DEC C");
                let half_c = self.cpu.regs[Regs::C as usize] & 0x0F;
                let half_one = 1 & 0x0F;
                self.set_h_flag(half_c < half_one);

                self.reg_dec_c(1);

                self.set_z_flag(self.cpu.regs[Regs::C as usize] == 0);
                self.set_n_flag(true);
                self.cpu.pc += 1;
                return 4;
            }
            0x0E => {
                println!("LD C,d8");
                self.cpu.pc += 1;
                self.cpu.regs[Regs::C as usize] = self.cpu.memory[self.cpu.pc];
                self.cpu.pc += 1;
                return 8;
            }
            0x11 => {
                println!("LD DE,d16");
                self.cpu.pc += 1;
                self.cpu.regs[Regs::D as usize] = rom[self.cpu.pc];
                self.cpu.pc += 1;
                self.cpu.regs[Regs::E as usize] = rom[self.cpu.pc];

                self.cpu.pc += 1;
                return 12;
            }
            0x20 => {
                println!("JR NR,r8");
                // com branch: 12
                // sem branch: 8
                if !self.get_z_flag() {
                    self.cpu.pc += 1;
                    // pura gambiarra
                    // todo: maybe fix this later
                    self.cpu.pc = (self.cpu.pc as i128
                        + (self.cpu.memory[self.cpu.pc] as i8) as i128)
                        as usize
                        + 1;
                    return 12;
                }
                self.cpu.pc += 2;
                return 8;
            }
            0x21 => {
                println!("LD HL,d16");
                // regs -> h l -- mem -> x y
                //         y x
                self.cpu.pc += 1;
                self.cpu.regs[Regs::L as usize] = self.cpu.memory[self.cpu.pc];
                self.cpu.pc += 1;
                self.cpu.regs[Regs::H as usize] = self.cpu.memory[self.cpu.pc];
                self.cpu.pc += 1;
                return 12;
            }
            0x2C => {
                println!("INC L");
                // check for half carry first of all
                let half_reg = self.cpu.regs[Regs::L as usize] & 0x0F;
                let half_inc = 1;

                self.reg_inc_l(1);

                self.set_n_flag(false);
                self.set_z_flag(self.cpu.regs[Regs::L as usize] == 0);
                self.set_h_flag(half_inc + half_reg > 0xF);
                self.cpu.pc += 1;
                return 4;
            }
            0x32 => {
                println!("LD (HL-),A");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[Regs::A as usize];
                self.cpu.regs[Regs::L as usize] = self.cpu.regs[Regs::L as usize].wrapping_sub(1);
                if self.cpu.regs[Regs::L as usize] == 255 {
                    // number wraped around
                    self.cpu.regs[Regs::H as usize] =
                        self.cpu.regs[Regs::H as usize].wrapping_sub(1);
                }
                self.cpu.pc += 1;
                return 8;
            }
            0x3C => {
                println!("INC A");
                // check for half carry first of all
                let half_reg = self.cpu.regs[Regs::A as usize] & 0x0F;
                let half_inc = 1;

                self.reg_inc_a(1);

                self.set_n_flag(false);
                self.set_z_flag(self.cpu.regs[Regs::A as usize] == 0);
                self.set_h_flag(half_inc + half_reg > 0xF);
                self.cpu.pc += 1;
                return 4;
            }
            0x3E => {
                println!("LD A,d8");
                self.cpu.pc += 1;
                self.cpu.regs[Regs::A as usize] =  self.cpu.memory[self.cpu.pc];
                self.cpu.pc += 1;
                return 8;
            }
            0x4A => {
                println!("LD C,D");
                self.cpu.regs[Regs::C as usize] = self.cpu.regs[Regs::D as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x4B => {
                println!("LD C,E");
                self.cpu.regs[Regs::C as usize] = self.cpu.regs[Regs::E as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x53 => {
                println!("LD D,E");
                self.cpu.regs[Regs::D as usize] = self.cpu.regs[Regs::E as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x55 => {
                println!("LD D,L");
                self.cpu.regs[Regs::D as usize] = self.cpu.regs[Regs::L as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x56 => {
                println!("LD D,(HL)");
                self.cpu.regs[Regs::D as usize] = self.cpu.memory[self.get_hl() as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x57 => {
                println!("LD D,A");
                self.cpu.regs[Regs::D as usize] = self.cpu.regs[Regs::A as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x58 => {
                println!("LD E,B");
                self.cpu.regs[Regs::E as usize] = self.cpu.regs[Regs::B as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x59 => {
                println!("LD E,C");
                self.cpu.regs[Regs::E as usize] = self.cpu.regs[Regs::C as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x5A => {
                println!("LD E,D");
                self.cpu.regs[Regs::E as usize] = self.cpu.regs[Regs::D as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x5C => {
                println!("LD E,H");
                self.cpu.regs[Regs::E as usize] = self.cpu.regs[Regs::H as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x6C => {
                println!("LD L,H");
                self.cpu.regs[Regs::L as usize] = self.cpu.regs[Regs::H as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x6E => {
                println!("LD L,(HL)");
                self.cpu.regs[Regs::L as usize] = self.cpu.memory[self.get_hl() as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x6F => {
                println!("LD L,A");
                self.cpu.regs[Regs::L as usize] = self.cpu.regs[Regs::A as usize];
                self.cpu.pc += 1;
                return 4;
            }
            0x71 => {
                println!("LD (HL),C");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[Regs::C as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x72 => {
                println!("LD (HL),D");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[Regs::D as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x73 => {
                println!("LD (HL),E");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[Regs::E as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x74 => {
                println!("LD (HL),H");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[Regs::H as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x75 => {
                println!("LD (HL),L");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[Regs::L as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0xAF => {
                println!("XOR A");
                self.cpu.regs[Regs::A as usize] = 0;
                self.cpu.pc += 1;
                return 4;
            }
            0xC3 => {
                println!("JMP a16");
                // and this is where I learnt the difference between big and small endian
                // now I just wonder where else have I not flipped the bytes where I should
                let mut new_address = 0;
                self.cpu.pc += 1;
                new_address |= rom[self.cpu.pc] as usize;
                self.cpu.pc += 1;
                new_address |= (rom[self.cpu.pc] as usize) << 8;
                self.cpu.pc = new_address;
                return 16;
            }
            0xF3 => {
                self.cpu.ime = false;
                self.cpu.pc += 1;
                return 4;
            }
            0x90 => {
                println!("SUB B");
                self.sub_reg_from_a(Regs::B);
                self.cpu.pc += 1;
                return 4;
            }
            0x91 => {
                println!("SUB C");
                self.sub_reg_from_a(Regs::C);
                self.cpu.pc += 1;
                return 4;
            }
            0x92 => {
                println!("SUB D");
                self.sub_reg_from_a(Regs::D);
                self.cpu.pc += 1;
                return 4;
            }
            0x93 => {
                println!("SUB E");
                self.sub_reg_from_a(Regs::E);
                self.cpu.pc += 1;
                return 4;
            }
            0x94 => {
                println!("SUB H");
                self.sub_reg_from_a(Regs::H);
                self.cpu.pc += 1;
                return 4;
            }
            0x95 => {
                println!("SUB L");
                self.sub_reg_from_a(Regs::L);
                self.cpu.pc += 1;
                return 4;
            }
            _ => todo!(
                "{:02X}\nPC: {:04X} | {}",
                rom[self.cpu.pc],
                self.cpu.pc,
                self.cpu.pc
            ),
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

        loop {
            self.compute(rom);
        }
    }
}
