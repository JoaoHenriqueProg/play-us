use core::panic;
use std::str::Bytes;

use crate::emulator::Emulator;
use eframe::egui::{self, Color32, RichText, Sense};

struct RamViewer {
    ram: [u8; 64 * 1024],
    address_to_look: String,
    starting_address: usize
}

impl Default for RamViewer {
    fn default() -> Self {
        Self {
            ram: [0; 64 * 1024],
            address_to_look: "".to_string(),
            starting_address: 0
        }
    }
}

impl RamViewer {
    fn new(ram: [u8; 64 * 1024]) -> Self {
        Self { ram, address_to_look: "".to_string(), starting_address: 0 }
    }
}

impl eframe::App for RamViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RAM Viewer");
            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            
            ui.vertical(|ui| {
                ui.label("Address");
                let mut i = self.starting_address;
                
                ui.text_edit_singleline(&mut self.address_to_look);
                if ui.button("Look up").clicked() {
                    // validate address
                    for c in self.address_to_look.chars() {
                        match usize::from_str_radix(&self.address_to_look, 16) {
                            Ok(mut found) => {
                                found -= found % 16;
                                self.starting_address = found;
                            },
                            Err(_) => return,
                        }
                    }
                }
                ui.label(RichText::new("      0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F").size(14.0).monospace());
                while i < 64 * 1024 {
                    ui.label(RichText::new(format!(
                        "{:04X}: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                        i,
                        self.ram[i],
                        self.ram[i + 1],
                        self.ram[i + 2],
                        self.ram[i + 3],
                        self.ram[i + 4],
                        self.ram[i + 5],
                        self.ram[i + 6],
                        self.ram[i + 7],
                        self.ram[i + 8],
                        self.ram[i + 9],
                        self.ram[i + 10],
                        self.ram[i + 11],
                        self.ram[i + 12],
                        self.ram[i + 13],
                        self.ram[i + 14],
                        self.ram[i + 15],
                    )).color(Color32::WHITE).monospace().size(14.0));
                    i += 16;
                }
            });

            // ui.image(egui::include_image!(
            //     "../../../crates/egui/assets/ferris.png"
            // ));
        });
    }
}

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
        println!("Set {:02X} at {:02X} (or {}) of hram", value, address, address + 0xFF00);
        
        // turns out this high area is used for a ton of flags that the hardware plays with
        match address {
            0x0000 => {
                // todo: capture inputs
                // for some goddamn reason, in gameboy, a not pressed button is 1
                // and a pressed button is 0
                if value == 0x20 { // bit 5 (select buttons) are the ones to be looked
                    self.cpu.memory[address + 0xFF00] = 0xEF;
                } else if value == 0x10 { // bit 4 (direction buttons) are the ones to be looked
                    self.cpu.memory[address + 0xFF00] = 0xDF;
                } else if value == 0x30 { // bit 4 and 5, no input is being captured
                    self.cpu.memory[address + 0xFF00] |= 0xF;
                } else {
                    panic!("god knows what");
                }
            }
            _ => println!("set_hram should check for {:04X}", address)
        }
    }
    fn get_hram(&mut self, address: usize) -> u8 {
        println!(
            "Got {:02X} at {:02X} (or {}) of hram",
            self.cpu.memory[address + 0xFF00],
            address,
            address + 0xFF00
        );
        return self.cpu.memory[address + 0xFF00];
    }

    fn inc_pair(&mut self, left: Regs, right: Regs) {
        self.cpu.regs[right] = self.cpu.regs[right].wrapping_add(1);
        if self.cpu.regs[right] == 0 {
            self.cpu.regs[left] = self.cpu.regs[left].wrapping_add(1);
        }
    }

    fn dec_pair(&mut self, left: Regs, right: Regs) {
        self.cpu.regs[right] = self.cpu.regs[right].wrapping_sub(1);
        if self.cpu.regs[right] == 255 {
            self.cpu.regs[left] = self.cpu.regs[left].wrapping_sub(1);
        }
    }

    fn pop_to_pair(&mut self, left: Regs, right: Regs) {
        self.cpu.sp += 2;
        self.cpu.regs[left] = self.cpu.memory[self.cpu.sp as usize + 1];
        self.cpu.regs[right] = self.cpu.memory[self.cpu.sp as usize];
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
    fn read(&self) -> u8 {
        self.cpu.memory[self.cpu.pc]
    }
    #[inline]
    fn read_next(&self, offset: usize) -> u8 {
        self.cpu.memory[self.cpu.pc + offset]
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
        println!("SP={:04X}", self.cpu.sp);
    }

    fn ram_viewer(&self) {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 720.0]),
            ..Default::default()
        };
        let rv = RamViewer::new(self.cpu.memory);
        eframe::run_native(
            "RAM viewer",
            options,
            Box::new(|cc| {
                // This gives us image support:
                // egui_extras::install_image_loaders(&cc.egui_ctx);

                Ok(Box::<RamViewer>::new(rv))
            }),
        )
        .unwrap();
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
        if !checked_functions.contains(&self.read()) {
            // panic!("{:02X}", self.read())
        }
        match self.read() {
            // 5B (LD E,E)
            0 | 0x5B => {
                println!("NOP");
                self.cpu.pc += 1;
                return 4;
            }
            0x01 => {
                println!("LD BC,d16");
                // regs -> b c -- mem -> x y
                //         y x
                self.cpu.regs[RegC] = self.read_next(1);
                self.cpu.regs[RegB] = self.read_next(2);
                self.cpu.pc += 3;
                return 12;
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
                self.cpu.regs[RegB] = self.read_next(1);
                self.cpu.pc += 2;
                return 8;
            }
            0x0B => {
                println!("DEC BC");
                self.dec_pair(RegB, RegC);
                self.cpu.pc += 1;
                return 8;
            }
            0x0C => {
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
                self.cpu.regs[RegC] = self.read_next(1);
                self.cpu.pc += 2;
                return 8;
            }
            0x0F => {
                println!("RRCA");
                self.set_c_flag(self.cpu.regs[RegA] & 1 == 1);
                self.cpu.regs[RegA] = self.cpu.regs[RegA].rotate_right(1);
                self.set_z_flag(false);
                self.set_n_flag(false);
                self.set_h_flag(false);
                self.cpu.pc += 1;
                return 4
            }
            // todo: to check
            0x11 => {
                println!("LD DE,d16");
                self.cpu.regs[RegE] = self.read_next(1);
                self.cpu.regs[RegD] = self.read_next(2);
                self.cpu.pc += 3;
                return 12;
            }
            0x12 => {
                println!("LD (DE),A");
                let address = 0 | (self.cpu.regs[RegD] as usize) << 8 | (self.cpu.regs[RegE] as usize);
                self.cpu.memory[address] = self.cpu.regs[RegA];
                self.cpu.pc += 1;
                return 8;
            }
            0x13 => {
                println!("INC DE");
                self.inc_pair(RegD, RegE);
                self.cpu.pc += 1;
                return 8;
            }
            0x16 => {
                println!("LD D,d8");
                self.cpu.regs[RegD] = self.read_next(1);
                self.cpu.pc += 2;
                return 8;
            }
            0x18 => {
                println!("JR a8");
                self.cpu.pc += 1;
                self.cpu.pc = (self.cpu.pc as i128 + (self.read() as i8) as i128) as usize + 1;
                return 12;
            }
            0x19 => {
                println!("ADD HL, DE");
                let hl = (self.cpu.regs[RegH] as u16) << 8 | self.cpu.regs[RegL] as u16;
                let de = (self.cpu.regs[RegD] as u16) << 8 | self.cpu.regs[RegE] as u16;
                // aparently the half flag takes in bit 11 and 12 as half in 16 bit math
                self.set_h_flag((hl & 0xFFF) + (de & 0xFFF) > 0xFFF);
                let result = hl + de;
                self.set_c_flag(result < hl && result < de);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                self.cpu.regs[RegH] = (result >> 8) as u8;
                self.cpu.regs[RegL] = (result & 0x00FF) as u8;
                return 8;
            }
            0x1A => {
                println!("LD A,(DE)");
                let de = (self.cpu.regs[RegD] as u16) << 8 | self.cpu.regs[RegE] as u16;
                self.cpu.regs[RegA] = self.cpu.memory[de as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x1C => {
                println!("INC E");
                self.op_inc_reg(RegE);
                self.cpu.pc += 1;
                return 4;
                
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
                self.cpu.pc = (self.cpu.pc as i128 + (self.read() as i8) as i128) as usize + 1;
                return 12;
            }
            0x21 => {
                println!("LD HL,d16");
                // regs -> h l -- mem -> x y
                //         y x
                // todo: check if this is correct later
                self.cpu.regs[RegL] = self.read_next(1);
                self.cpu.regs[RegH] = self.read_next(2);
                self.cpu.pc += 3;
                return 12;
            }
            0x22 => {
                println!("LD (HL+),A");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegA];
                self.inc_pair(RegH, RegL);
                self.cpu.pc += 1;
                return 8;
            }
            0x23 => {
                println!("INC HL");
                self.inc_pair(RegH, RegL);
                self.cpu.pc += 1;
                return 8;
            }
            0x28 => {
                println!("JR Z,r8");
                // com branch: 12
                // sem branch: 8
                if !self.get_z_flag() {
                    self.cpu.pc += 2;
                    return 8;
                }
                self.cpu.pc += 1;
                self.cpu.pc = (self.cpu.pc as i128 + (self.read() as i8) as i128) as usize + 1;
                return 12;
            }
            0x2A => {
                println!("LD A,(HL+)");
                let mut address = 0;
                address |= ((self.get_hl() & 0x0F) as usize) << 8; // putting l left
                address |= ((self.get_hl() & 0xF0) as usize) >> 8; // putting h right
                self.cpu.regs[RegA] = self.cpu.memory[self.get_hl() as usize]; // SINCE WHEN WAS THIS LINE COMMENTED
                // NOTE: ADDRESSING MEMORY WITH HL DOESN'T TREAT IT LIKE LITTLE ENDIAN
                self.inc_pair(RegH, RegL);
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
            0x2F => {
                println!("CPL");
                // check for half carry first of all
                self.cpu.regs[RegA] = !self.cpu.regs[RegA];
                self.set_n_flag(true);
                self.set_h_flag(true);
                self.cpu.pc += 1;
                return 4;
            }
            0x31 => {
                println!("LD SP,d16");
                let mut value_d16 = 0;
                value_d16 |= self.read_next(1) as usize;
                value_d16 |= (self.read_next(2) as usize) << 8;
                self.cpu.sp = value_d16 as u16;
                self.cpu.pc += 3;
                return 12;
            }
            0x32 => {
                println!("LD (HL-),A");
                self.cpu.memory[self.get_hl() as usize] = self.cpu.regs[RegA]; // SINCE WHEN WAS THIS LINE COMMENTED
                self.dec_pair(RegH, RegL);
                self.cpu.pc += 1;
                return 8;
            }
            0x34 => {
                println!("INC (HL)");
                let cur_val = self.cpu.memory[self.get_hl() as usize];
                let half_reg = cur_val & 0x0F;
                let half_inc = 1;

                self.cpu.memory[self.get_hl() as usize] = cur_val.wrapping_add(1);

                self.set_n_flag(false);
                self.set_z_flag(self.cpu.memory[self.get_hl() as usize] == 0);
                self.set_h_flag(half_inc + half_reg > 0xF);
                self.cpu.pc += 1;
                return 12;
            }
            0x35 => {
                println!("DEC (HL)");
                let cur_val = self.cpu.memory[self.get_hl() as usize];
                let half_val = cur_val & 0x0F;
                let half_one = 1;
                self.set_h_flag(half_val < half_one);

                self.cpu.memory[self.get_hl() as usize] = cur_val.wrapping_sub(1);

                self.set_z_flag(self.cpu.memory[self.get_hl() as usize] == 0);
                self.set_n_flag(true);
                self.cpu.pc += 1;
                return 12
            }
            0x36 => {
                println!("LD (HL),d8");
                self.cpu.memory[self.get_hl() as usize] = self.read_next(1);
                self.cpu.pc += 2;
                return 12;
            }
            0x3C => {
                println!("INC A");
                self.op_inc_reg(RegA);
                self.cpu.pc += 1;
                return 4;
            }
            0x3D => {
                println!("DEC A");
                self.op_dec_reg(RegA);
                self.cpu.pc += 1;
                return 4;
            }
            0x3E => {
                println!("LD A,d8");
                self.cpu.regs[RegA] = self.read_next(1);
                self.cpu.pc += 2;
                return 8;
            }
            0x47 => {
                println!("LD B,A");
                self.cpu.regs[RegB] = self.cpu.regs[RegA];
                self.cpu.pc += 1;
                return 4;
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
            0x4F => {
                println!("LD C,A");
                self.cpu.regs[RegC] = self.cpu.regs[RegA];
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
            0x5E => {
                println!("LD E,(HL)");
                self.cpu.regs[RegE] = self.cpu.memory[self.get_hl() as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x5F => {
                println!("LD E,A");
                self.cpu.regs[RegE] = self.cpu.regs[RegA];
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
            0x78 => {
                println!("LD A,B");
                self.cpu.regs[RegA] = self.cpu.regs[RegB];
                self.cpu.pc += 1;
                return 4;
            }
            0x79 => {
                println!("LD A,C");
                self.cpu.regs[RegA] = self.cpu.regs[RegC];
                self.cpu.pc += 1;
                return 4;
            }
            0x7C => {
                println!("LD A,H");
                self.cpu.regs[RegA] = self.cpu.regs[RegH];
                self.cpu.pc += 1;
                return 4;
            }
            0x7E => {
                println!("LD A,(HL)");
                self.cpu.regs[RegA] = self.cpu.memory[self.get_hl() as usize];
                self.cpu.pc += 1;
                return 8;
            }
            0x87 => {
                println!("ADD A,A");
                self.set_h_flag((self.cpu.regs[RegA] & 0xF) + (self.cpu.regs[RegA] & 0xF) > 0xF);
                let result = self.cpu.regs[RegA].wrapping_add(self.cpu.regs[RegA]);
                self.set_c_flag(result < self.cpu.regs[RegA]);
                self.cpu.regs[RegA] = result;
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                return 4
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
            0xA1 => {
                println!("AND B");
                self.cpu.regs[RegA] &= self.cpu.regs[RegB];
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_c_flag(false);
                self.set_h_flag(true);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                return 4;
            }
            0xA7 => {
                println!("AND A");
                // todo: maybe this could be removed
                self.cpu.regs[RegA] &= self.cpu.regs[RegA];
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_c_flag(false);
                self.set_h_flag(true);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                return 4;
            }
            0xA9 => {
                println!("XOR C");
                self.cpu.regs[RegA] ^= self.cpu.regs[RegC];
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_c_flag(false);
                self.set_h_flag(false);
                self.set_n_flag(false);
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
            0xB0 => {
                println!("OR B");
                self.cpu.regs[RegA] = self.cpu.regs[RegA] | self.cpu.regs[RegB];
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_c_flag(false);
                self.set_h_flag(false);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                return 4;
            }
            0xB1 => {
                println!("OR C");
                self.cpu.regs[RegA] = self.cpu.regs[RegA] | self.cpu.regs[RegC];
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_c_flag(false);
                self.set_h_flag(false);
                self.set_n_flag(false);
                self.cpu.pc += 1;
                return 4;
            }
            0xC1 => {
                println!("POP BC");
                self.pop_to_pair(RegB, RegC);
                self.cpu.pc += 1;
                return 12;
            }
            0xC0 => {
                println!("RET NZ");
                if self.get_z_flag() {
                    self.cpu.pc += 1;
                    return 8;
                }
                let mut new_address = 0;
                self.cpu.sp += 2;
                new_address |= self.cpu.memory[self.cpu.sp as usize] as usize;
                new_address |= (self.cpu.memory[self.cpu.sp as usize + 1] as usize) << 8;
                self.cpu.pc = new_address;
                return 20;
            }
            0xC3 => {
                println!("JMP a16");
                // and this is where I learnt the difference between big and small endian
                // now I just wonder where else have I not flipped the bytes where I should
                let mut new_address = 0;
                new_address |= self.read_next(1) as usize;
                new_address |= (self.read_next(2) as usize) << 8;
                self.cpu.pc = new_address as usize;
                return 16;
            }
            0xC5 => {
                println!("PUSH BC");
                println!("{:4X}", self.cpu.sp);
                self.cpu.memory[self.cpu.sp as usize] = self.cpu.regs[RegC];
                self.cpu.memory[self.cpu.sp as usize + 1] = self.cpu.regs[RegB];
                self.cpu.sp -= 2;
                self.cpu.pc += 1;
                return 16;
            }
            0xC8 => {
                println!("RET Z");
                if !self.get_z_flag() {
                    self.cpu.pc += 1;
                    return 8
                }
                let mut new_address = 0;
                self.cpu.sp += 2;
                new_address |= self.cpu.memory[self.cpu.sp as usize] as usize;
                new_address |= (self.cpu.memory[self.cpu.sp as usize + 1] as usize) << 8;
                self.cpu.pc = new_address;
                return 20;
            }
            0xC9 => {
                println!("RET");
                let mut new_address = 0;
                self.cpu.sp += 2;
                new_address |= self.cpu.memory[self.cpu.sp as usize] as usize;
                new_address |= (self.cpu.memory[self.cpu.sp as usize + 1] as usize) << 8;
                self.cpu.pc = new_address;
                return 16;
            }
            0xCA => {
                println!("JP Z,a16");
                // com branch: 16
                // sem branch: 12
                if !self.get_z_flag() {
                    self.cpu.pc += 3;
                    return 12;
                }
                let mut new_address = 0;
                new_address |= self.read_next(1) as usize;
                new_address |= (self.read_next(2) as usize) << 8;
                self.cpu.pc = new_address;
                return 16;
            }
            0xCB => {
                print!("CB: ");
                self.cpu.pc += 1;
                return self.compute_cb();
            }
            0xCD => {
                println!("CALL a16");
                let mut next_instruction = (self.cpu.pc & 0xFFFF) as u16 + 3; // offsetting to the next instruction
                // stores in little endian
                self.cpu.memory[self.cpu.sp as usize] = (next_instruction & 0x00FF) as u8;
                self.cpu.memory[self.cpu.sp as usize + 1] = (next_instruction >> 8) as u8;

                let mut new_address = 0;
                new_address |= self.read_next(1) as usize;
                new_address |= (self.read_next(2) as usize) << 8;
                
                self.cpu.sp -= 2; // apparently the stack is "upside down"
                self.cpu.pc =  new_address;
                return 24
            }
            0xD1 => {
                println!("POP DE");
                self.pop_to_pair(RegD, RegE);
                self.cpu.pc += 1;
                return 12;
            }
            0xD5 => {
                // might break things soon
                // todo: keep here in mind
                println!("PUSH DE");
                self.cpu.memory[self.cpu.sp as usize] = self.cpu.regs[RegE];
                self.cpu.memory[self.cpu.sp as usize + 1] = self.cpu.regs[RegD];
                self.cpu.sp -= 2;
                self.cpu.pc += 1;
                return 16
            }
            0xD9 => {
                println!("RETI");
                self.cpu.ime = true;
                
                let mut new_address = 0;
                self.cpu.sp += 2;
                new_address |= self.cpu.memory[self.cpu.sp as usize] as usize;
                new_address |= (self.cpu.memory[self.cpu.sp as usize + 1] as usize) << 8;
                self.cpu.pc = new_address;
                return 16
            }
            0xE1 => {
                println!("POP HL");
                self.pop_to_pair(RegH, RegL);
                self.cpu.pc += 1;
                return 12;
            }
            0xE2 => {
                println!("LD (C),A");
                self.cpu.memory[self.cpu.regs[RegC] as usize + 0xFF00] = self.cpu.regs[RegA];
                self.cpu.pc += 1;
                return 8;
            }
            0xE5 => {
                // might break things soon
                // todo: keep here in mind
                println!("PUSH HL");
                self.cpu.memory[self.cpu.sp as usize] = self.cpu.regs[RegL];
                self.cpu.memory[self.cpu.sp as usize + 1] = self.cpu.regs[RegH];
                self.cpu.sp -= 2;
                self.cpu.pc += 1;
                return 16
            }
            0xE6 => {
                println!("AND d8");
                self.cpu.regs[RegA] &= self.read_next(1);
                self.set_z_flag(self.cpu.regs[RegA] == 0);
                self.set_n_flag(false);
                self.set_h_flag(true);
                self.set_c_flag(false);
                self.cpu.pc += 2;
                return 8;
            }
            0xE9 => {
                println!("JP (HL)");
                self.cpu.pc = self.get_hl() as usize;
                return 8;
            }
            0xEA => {
                println!("LD (a16),A");
                let mut address = 0;
                address |= self.read_next(1) as usize;
                address |= (self.read_next(2) as usize) << 8;
                self.cpu.memory[address as usize] = self.cpu.regs[RegA];
                self.cpu.pc += 3;
                return 16;
            }
            0xEF => {
                println!("RST 28H");
                let next_instruction = (self.cpu.pc & 0xFFFF) as u16 + 1; // offsetting to the next instruction
                // stores in little endian
                self.cpu.memory[self.cpu.sp as usize] = (next_instruction & 0x00FF) as u8;
                self.cpu.memory[self.cpu.sp as usize + 1] = (next_instruction >> 8) as u8;
                self.cpu.sp -= 2;
                self.cpu.pc = 0x0028;
                return 16
            }
            0xE0 => {
                println!("LDH (a8),A");
                self.set_hram(self.read_next(1) as usize, self.cpu.regs[RegA]);
                self.cpu.pc += 2;
                return 12;
            }
            0xF0 => {
                println!("LDH A,(a8)");
                self.cpu.regs[RegA] = self.get_hram(self.read_next(1) as usize);
                // I spent almost a whole day trying to find out why where the game in an infinite loop
                // Till I had the great idea of using the concept of "searching online"
                // Turns out the game keeps waiting for the game to draw, which is when 0xFF44 (the y lcd counter)
                // Is 148 (or whatever it is in hex)
                self.cpu.pc += 2;
                return 12;
            }
            0xF1 => {
                println!("POP AF");
                self.pop_to_pair(RegA, RegF);
                self.cpu.pc += 1;
                return 12;
            }
            0xF3 => {
                println!("DI");
                self.cpu.ime = false;
                self.cpu.pc += 1;
                return 4;
            }
            0xF5 => {
                println!("PUSH AF");
                self.cpu.memory[self.cpu.sp as usize] = self.cpu.regs[RegF];
                self.cpu.memory[self.cpu.sp as usize + 1] = self.cpu.regs[RegA];
                self.cpu.sp -= 2;
                self.cpu.pc += 1;
                return 16;
            }
            0xFA => {
                println!("LD A,(a16)");
                let mut address = 0;
                address |= self.read_next(1) as usize;
                address |= (self.read_next(2) as usize) << 8;
                self.cpu.regs[RegA] = self.cpu.memory[address];
                self.cpu.pc += 3;
                return 16;
            }
            0xFB => {
                println!("EI");
                self.cpu.ime = true;
                self.cpu.pc += 1;
                return 4;
            }
            0xFE => {
                println!("CP d8");
                let value = self.read_next(1);
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
                self.ram_viewer();
                todo!(
                    "{:02X}\nPC: {:04X} | {}",
                    self.read(),
                    self.cpu.pc,
                    self.cpu.pc
                )
            }
        }
    }
    fn compute_cb(&mut self) -> u64{
        // there's a second one :)
        match self.read() {
            0x30 | 0x31 | 0x32 | 0x33 | 0x34 | 0x35 | 0x36 | 0x37 => {
                if self.read() >= 0x30 && self.read() <= 0x35 { // regs b to l
                    println!("SWAP R"); // todo: name correct reg
                    let cur_reg = self.cpu.regs[(self.read() & 0x0F) as usize + 1];
                    let new_reg = 0 | (cur_reg << 4) | (cur_reg >> 4);
                    self.cpu.regs[(self.read() & 0x0F) as usize + 1] = new_reg;
                    self.set_z_flag(self.cpu.regs[(self.read() & 0x0F) as usize + 1] == 0);
                    panic!("check if this is correct once used");
                } else if self.read() == 0x36 {
                    todo!("BC SWAP (HL)")
                } else {
                    println!("SWAP A");
                    let cur_reg = self.cpu.regs[RegA];
                    let new_reg = 0 | (cur_reg << 4) | (cur_reg >> 4);
                    self.cpu.regs[RegA] = new_reg;
                    self.set_z_flag(RegA == 0);
                }
                self.set_n_flag(false);
                self.set_h_flag(false);
                self.set_c_flag(false);
                self.cpu.pc += 1;
                return 8
            }
            // RES instructions
            0x87 => {
                println!("RES 0,A");
                self.cpu.regs[RegA] &= !1;
                self.cpu.pc += 1;
                return 8
            }
            _ => {
                self.print_regs();
                todo!(
                    "CB: {:02X}\nPC: {:04X} | {}",
                    self.read(),
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

        let mut cycles_count = 0;
        // let mut steps = 4;
        loop {
            if self.cpu.ime {
                // https://gbdev.io/pandocs/Interrupts.html#ffff--ie-interrupt-enable
                let interrupts = self.cpu.memory[0xff0f] & self.cpu.memory[0xffff];
                
                // no interrupts requested
                if interrupts == 0 {}
                // vblank interrupt
                else if interrupts & 1 == 1 {
                    let cur_instruction = self.cpu.pc;
                    self.cpu.memory[self.cpu.sp as usize] = (cur_instruction & 0x00FF) as u8;
                    self.cpu.memory[self.cpu.sp as usize + 1] = (cur_instruction >> 8) as u8;
                    self.cpu.sp -= 2;
                    
                    self.cpu.pc = 0x40;
                    
                    self.cpu.ime = false;
                    
                    self.cpu.memory[0xff0f] &= !0x0001;
                } else {
                    todo!(" unhandled interrupt {}", interrupts)
                }
            }
            // if steps == 0 {
            //     self.print_regs();
            //     break;
            // }
            cycles_count += self.compute(rom);
            // if self.cpu.pc == 0x100 {
            //     self.print_regs();
            //     break;
            // }
            // steps -= 1;

            if cycles_count > 456 {
                // draw scan line
                self.cpu.memory[0xff44] += 1;
                if self.cpu.memory[0xff44] == 144 {
                    // request vblink interrupt
                    self.cpu.memory[0xff0f] |= 1;
                }
                if self.cpu.memory[0xff44] == 154 {
                    self.cpu.memory[0xff44] = 0;
                }
                cycles_count -= 456;
            }
        }
    }
}
