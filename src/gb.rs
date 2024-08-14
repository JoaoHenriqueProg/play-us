use crate::emulator::Emulator;

struct Cpu {
    bus_8: u8,
    bus_16: u16,
    memory: [u8; 1024 * 64],
    regs: [u8; 8],
    // AF, BC, DE, HL, by the gods what does it mean why this order
    pc: usize,
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            bus_8: 0,
            bus_16: 0,
            memory: [0; 1024 * 64],
            regs: [0; 8],
            pc: 0x100,
        }
    }
}

pub struct GameBoyEmulator {
    cpu: Cpu,
}

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
    fn reg_inc_a(&mut self, to_inc: u8) {
        self.cpu.regs[Regs::A as usize] = self.cpu.regs[Regs::A as usize].wrapping_add(to_inc);
    }
    #[inline]
    fn reg_dec_b(&mut self, to_dec: u8) {
        self.cpu.regs[Regs::B as usize] = self.cpu.regs[Regs::A as usize].wrapping_sub(to_dec);
    }

    // I know I probably shouldn't start directly implement opcodes, but preguicinha of doing
    // the game boy architecture and stuff
    fn compute(&mut self, rom: &[u8]) -> u64 {
        // returns the cpu cycles it takes, so in the future I can implement real cpu bottleneck
        print!("{:04X}: ", self.cpu.pc);
        match rom[self.cpu.pc] {
            0 => {
                println!("NOP");
                self.cpu.pc += 1;
                return 4;
            }
            0x05 => {
                // check for half carry
                let half_b = self.cpu.regs[Regs::B as usize] & 0x0F;
                let half_one = 1 & 0x0F;
                self.set_h_flag(half_one > half_b);

                self.reg_dec_b(1);

                self.set_z_flag(self.cpu.regs[Regs::B as usize] == 0);
                self.set_n_flag(true);
                self.cpu.pc += 1;
                return 4;
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
        loop {
            self.compute(rom);
        }
    }
}
