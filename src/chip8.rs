use crate::emulator::Emulator;

struct Cpu {
    regs: [u8; 16],
    ip: usize,
    mem_address: u16 // TODO: Funções para monipulação, segundo a wikipedia, é um número com 12 bits, então o valor máximo seria 4095
}

pub struct Chip8Emulator {
    cpu: Cpu,
    memory: [u8; 4096],
    stack: [u16; 16],
    sp: usize
}

impl Emulator for Chip8Emulator {
    fn run(&mut self, rom: &[u8]) {
        loop {
            let hex1 = format!("{:02X}", rom[self.cpu.ip]).chars().nth(0).unwrap();
            let hex2 = format!("{:02X}", rom[self.cpu.ip]).chars().nth(1).unwrap();
            let hex3 = format!("{:02X}", rom[self.cpu.ip + 1]).chars().nth(0).unwrap();
            let hex4 = format!("{:02X}", rom[self.cpu.ip + 1]).chars().nth(1).unwrap();
            match (hex1, hex2, hex3, hex4) {
                ('0', '0', 'E', '0') => {
                    println!("[TODO]: Clear screen");
                    self.cpu.ip += 2;
                }
                ('1', n1, n2, n3) => {
                    self.cpu.ip = usize::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16).unwrap();
                }
                ('2', n1, n2, n3) => {
                    self.stack[self.sp] = self.cpu.ip as u16;
                    self.sp += 1;
                    self.cpu.ip = usize::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16).unwrap();
                    // * divided by two because a instruction is two bytes?
                }
                ('4', r, n1, n2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let value_to_check = u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                    if self.cpu.regs[reg_i] != value_to_check {
                        self.cpu.ip += 2;
                    }
                    self.cpu.ip += 2;
                }
                ('6', r, n1, n2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let value_to_set = u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                    self.cpu.regs[reg_i] = value_to_set;
                    self.cpu.ip += 2;
                }
                ('A', n1, n2, n3) => {
                    self.cpu.mem_address = u16::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16).unwrap();
                    self.cpu.ip += 2;
                }
                ('F', r, '5', '5') => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    for i in 0..=reg_i {
                        self.memory[self.cpu.mem_address as usize + i] = self.cpu.regs[i];
                    }
                    self.cpu.ip += 2;
                }
                _ => todo!("HEX: {}{}{}{}\nIP: {}", hex1, hex2, hex3, hex4, self.cpu.ip)
            }
        }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: [0; 16],
            ip: 0,
            mem_address: 0
        }
    }
}

impl Chip8Emulator {
    pub fn new() -> Chip8Emulator {
        Chip8Emulator {
            cpu: Cpu::new(),
            memory: [0; 4096],
            stack: [0; 16],
            sp: 0,

        }
    }
}