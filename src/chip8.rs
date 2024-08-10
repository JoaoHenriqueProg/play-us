use crate::emulator::Emulator;
use rand::Rng;

struct Cpu {
    regs: [u8; 16],
    ip: usize,
    mem_address: u16 // TODO: Funções para monipulação, segundo a wikipedia, é um número com 12 bits, então o valor máximo seria 4095
}

pub struct Chip8Emulator {
    cpu: Cpu,
    memory: [u8; 4096],
    stack: [u16; 16],
    sp: usize,
    dt: usize,
    st: usize
}

impl Emulator for Chip8Emulator {
    fn run(&mut self, rom: &[u8]) {
        loop {
            // if self.cpu.ip % 2 != 0 {
            //     panic!("God please don't") // Space invader reached here, which means something is wrong
            // }

            let hex1 = format!("{:02X}", rom[self.cpu.ip]).chars().nth(0).unwrap();
            let hex2 = format!("{:02X}", rom[self.cpu.ip]).chars().nth(1).unwrap();
            let hex3 = format!("{:02X}", rom[self.cpu.ip + 1]).chars().nth(0).unwrap();
            let hex4 = format!("{:02X}", rom[self.cpu.ip + 1]).chars().nth(1).unwrap();
            match (hex1, hex2, hex3, hex4) {
                ('0', '0', 'E', '0') => {
                    println!("[TODO]: Clear screen");
                }
                ('0', '0', 'E', 'E') => {
                    println!("[RET]");
                    self.sp -= 1;
                    self.cpu.ip = self.stack[self.sp] as usize;
                }
                ('1', n1, n2, n3) => {
                    self.cpu.ip = usize::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16).unwrap() - 512;
                    // The game code should be in memory, so we subtract its offset
                    // Maybe will have to move the game code into memory in the future
                    println!("[JUMP]: {:?}", self.cpu.ip);
                    continue // does not increment counter
                }
                ('2', n1, n2, n3) => {
                    self.stack[self.sp] = self.cpu.ip as u16;
                    self.sp += 1;
                    self.cpu.ip = usize::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16).unwrap() - 512;
                    // * divided by two because a instruction is two bytes?
                    continue // does not increment counter
                }
                ('3', r, n1, n2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    println!("[IINE]: V{:?}", r); // Ignore if not equal
                    let value_to_check = u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                    if self.cpu.regs[reg_i] == value_to_check {
                        self.cpu.ip += 2;
                        println!("[INFO]: Ignored.");
                    }
                }
                ('4', r, n1, n2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let value_to_check = u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                    println!("[IIE ]: V{} == {}", reg_i, value_to_check); // Ignore if equal
                    if self.cpu.regs[reg_i] != value_to_check {
                        self.cpu.ip += 2;
                        println!("[INFO]: Ignored.");
                    }
                }
                ('6', r, n1, n2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let value_to_set = u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                    self.cpu.regs[reg_i] = value_to_set;
                    println!("[MSET]: {:?} at V{}", value_to_set, reg_i);
                }
                ('7', r, n1, n2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let value_to_add = u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                    self.cpu.regs[reg_i] += value_to_add;
                    println!("[INCR]: V{reg_i} += {value_to_add}");
                }
                ('8', r1, r2, '0') => {
                    let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                    let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                    println!("[CPXY]: V{r1} = V{r2} = {}", self.cpu.regs[reg_i2]);
                    self.cpu.regs[reg_i1] = self.cpu.regs[reg_i2];
                }
                ('8', r1, r2, '2') => {
                    let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                    let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                    println!("[ANDR]: V{r1} = V{r1} & V{r2} = {:08b} & {:08b} = {:08b}", self.cpu.regs[reg_i1], self.cpu.regs[reg_i2], self.cpu.regs[reg_i1] & self.cpu.regs[reg_i2]);
                    self.cpu.regs[reg_i1] = self.cpu.regs[reg_i1] & self.cpu.regs[reg_i2];
                }
                ('8', r1, r2, '3') => {
                    let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                    let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                    println!("[XORR]: V{r1} = V{r1} ^ V{r2} = {:08b} ^ {:08b} = {:08b}", self.cpu.regs[reg_i1], self.cpu.regs[reg_i2], self.cpu.regs[reg_i1] ^ self.cpu.regs[reg_i2]);
                    self.cpu.regs[reg_i1] = self.cpu.regs[reg_i1] ^ self.cpu.regs[reg_i2];
                }
                ('8', r1, _r2, '6') => {
                    let reg_i = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                    if self.cpu.regs[reg_i] & 0b1 == 1 {
                        self.cpu.regs[15] = 1;
                    } else {
                        self.cpu.regs[15] = 0;
                    }
                    self.cpu.regs[reg_i] = self.cpu.regs[reg_i] >> 1;
                    println!("[SRCL]: V{r1} VF = {}", self.cpu.regs[15]);
                }
                ('8', r1, _r2, 'E') => {
                    let reg_i = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                    if self.cpu.regs[reg_i] & 0b10000000 == 0b10000000 {
                        self.cpu.regs[15] = 1;
                    } else {
                        self.cpu.regs[15] = 0;
                    }
                    self.cpu.regs[reg_i] = self.cpu.regs[reg_i] << 1;
                    println!("[SLCM]: V{r1} VF = {}", self.cpu.regs[15]);
                }
                ('A', n1, n2, n3) => {
                    self.cpu.mem_address = u16::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16).unwrap();
                    println!("[MPST]: {}", self.cpu.mem_address);
                }
                ('C', r, k1, k2) => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let mut rng = rand::thread_rng();
                    let value_to_and = u8::from_str_radix(format!("{}{}", k1, k2).as_str(), 16).unwrap();
                    let n: u8 = rng.gen::<u8>() & value_to_and;
                    self.cpu.regs[reg_i] = n;
                    println!("[RAND]: {r}");
                }
                ('E', k, '9', 'E') => {
                    let _key_i = usize::from_str_radix(k.to_string().as_str(), 16).unwrap();
                    println!("[SIKP]: To be completed, key not pressed");
                    println!("[TODO]: Check if key is pressed");
                }
                ('F', n, '1', '8') => {
                    self.st = usize::from_str_radix(n.to_string().as_str(), 16).unwrap();
                    println!("[STST]: Sound timer = {}", self.st);
                }
                ('F', r, '1', 'E') => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    self.cpu.mem_address += self.cpu.regs[reg_i] as u16;
                    println!("[ADDI]: {} from V{reg_i}", self.cpu.regs[reg_i]);
                }
                ('F', r, '3', '3') => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    let mem_address = self.cpu.mem_address as usize;
                    let to_text = format!("{:03}", self.cpu.regs[reg_i]);
                    self.memory[mem_address] = to_text.chars().nth(0).unwrap().to_digit(10).unwrap() as u8;
                    self.memory[mem_address + 1] = to_text.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                    self.memory[mem_address + 2] = to_text.chars().nth(2).unwrap().to_digit(10).unwrap() as u8;
                    println!("[DBCD]: V{reg_i} to {}", self.cpu.mem_address);
                    println!("  [INFO]: Value = {}, in memory = {} {} {}", self.cpu.regs[reg_i], self.memory[mem_address], self.memory[mem_address + 1], self.memory[mem_address + 2]);
                }
                ('F', r, '5', '5') => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    for i in 0..=reg_i {
                        self.memory[self.cpu.mem_address as usize + i] = self.cpu.regs[i];
                    }
                    println!("[RDMP]: V{reg_i} at {}", self.cpu.mem_address);
                }
                ('F', r, '6', '5') => {
                    let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                    for i in 0..=reg_i {
                        // self.memory[self.cpu.mem_address as usize + i] = self.cpu.regs[i];
                        self.cpu.regs[i] = self.memory[(self.cpu.mem_address + i as u16) as usize];
                    }
                    println!("[RRD ]: To V{reg_i} at {}", self.cpu.mem_address);
                }
                _ => todo!("HEX: {}{}{}{}\nIP: {}", hex1, hex2, hex3, hex4, self.cpu.ip)
            }
            self.cpu.ip += 2;
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
            dt: 0,
            st: 0,
        }
    }
}