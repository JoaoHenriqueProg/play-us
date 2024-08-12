use std::time::Instant;

use crate::video;
use crate::{emulator::Emulator, video::Screen};
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{self, Color, PixelFormatEnum};
use sdl2::rect::Rect;

struct Cpu {
    regs: [u8; 16],
    ip: usize,
    mem_address: u16, // TODO: Funções para monipulação, segundo a wikipedia, é um número com 12 bits, então o valor máximo seria 4095
}

pub struct Chip8Emulator {
    cpu: Cpu,
    memory: [u8; 4096],
    stack: [u16; 16],
    sp: usize,
    dt: usize,
    st: usize,
}

impl Emulator for Chip8Emulator {
    fn run(&mut self, rom: &[u8]) {
        let mut screen = Screen::new();
        let mut screen_bits = [false; 64 * 32];

        self.memory[512..512 + rom.len()].copy_from_slice(&rom);
        let mut cur_pressed_keys = [false; 16];
        // let mut steps = 19;
        let mut timers_timer = Instant::now();
        'main_loop: loop {
            // if self.cpu.ip % 2 != 0 {
            //     panic!("God please don't") // Space invader reached here, which means something is wrong
            // }

            // steps -= 1;
            // if steps == 0  {
            //     println!("{}", self.cpu.regs[0]);
            //     break
            // }
            if timers_timer.elapsed().as_millis() > 1000 / 60 {
                // the timers go down one each 1/60 of a second
                self.dt = self.dt.saturating_sub(1);
                self.st = self.st.saturating_sub(1);
                timers_timer = Instant::now();
            }

            for event in screen.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main_loop,
                    Event::KeyDown { keycode, .. } => {
                        if let Some(keycode) = keycode {
                            match keycode {
                                Keycode::NUM_1 => cur_pressed_keys[1] = true,
                                Keycode::NUM_2 => cur_pressed_keys[2] = true,
                                Keycode::NUM_3 => cur_pressed_keys[3] = true,
                                Keycode::Q => cur_pressed_keys[4] = true,
                                Keycode::W => cur_pressed_keys[5] = true,
                                Keycode::E => cur_pressed_keys[6] = true,
                                Keycode::A => cur_pressed_keys[7] = true,
                                Keycode::S => cur_pressed_keys[8] = true,
                                Keycode::D => cur_pressed_keys[9] = true,
                                Keycode::X => cur_pressed_keys[0] = true,
                                Keycode::Z => cur_pressed_keys[10] = true,
                                Keycode::C => cur_pressed_keys[11] = true,
                                Keycode::NUM_4 => cur_pressed_keys[12] = true,
                                Keycode::R => cur_pressed_keys[13] = true,
                                Keycode::F => cur_pressed_keys[14] = true,
                                Keycode::V => cur_pressed_keys[15] = true,
                                _ => {}
                            }
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(keycode) = keycode {
                            match keycode {
                                Keycode::NUM_1 => cur_pressed_keys[1] = false,
                                Keycode::NUM_2 => cur_pressed_keys[2] = false,
                                Keycode::NUM_3 => cur_pressed_keys[3] = false,
                                Keycode::Q => cur_pressed_keys[4] = false,
                                Keycode::W => cur_pressed_keys[5] = false,
                                Keycode::E => cur_pressed_keys[6] = false,
                                Keycode::A => cur_pressed_keys[7] = false,
                                Keycode::S => cur_pressed_keys[8] = false,
                                Keycode::D => cur_pressed_keys[9] = false,
                                Keycode::X => cur_pressed_keys[0] = false,
                                Keycode::Z => cur_pressed_keys[10] = false,
                                Keycode::C => cur_pressed_keys[11] = false,
                                Keycode::NUM_4 => cur_pressed_keys[12] = false,
                                Keycode::R => cur_pressed_keys[13] = false,
                                Keycode::F => cur_pressed_keys[14] = false,
                                Keycode::V => cur_pressed_keys[15] = false,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            // println!("KEYS PRESSED: {:?}", cur_pressed_keys);

            screen.canvas.set_draw_color(Color::BLACK);
            screen.canvas.clear();
            let mut rects = vec![];
            for pixel_y in 0..32 {
                for pixel_x in 0..64 {
                    // let clr = match screen_bits[pixel_x + pixel_y * 64] {
                    //     true => Color::WHITE,
                    //     false => Color::BLACK,
                    // };

                    // screen.canvas.set_draw_color(clr);
                    if screen_bits[pixel_x + pixel_y * 64] {
                        let rect = Rect::new(pixel_x as i32 * 8, pixel_y as i32 * 8, 8, 8);
                        // if clr.r == 255 {
                        //     println!("{:?} at {:?}", clr, rect);
                        // }
                        rects.push(rect);
                    }
                }
            }
            screen.canvas.set_draw_color(Color::WHITE);
            match screen.canvas.fill_rects(&rects) {
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            }
            screen.canvas.present();
            // for pixel_y in 0..32 {
            //     for pixel_x in 0..64 {
            //         let clr = match screen_bits[pixel_x + pixel_y * 64] {
            //             true => 1,
            //             false => 0,
            //         };

            //         print!("{clr}");
            //     }
            //     println!();
            // }

            
            print!("{:04X}: ", self.cpu.ip + 512);
            let frame_start = Instant::now();
            
            for _ in 0..750 / 60 {
                let hex1 = format!("{:02X}", rom[self.cpu.ip]).chars().nth(0).unwrap();
                let hex2 = format!("{:02X}", rom[self.cpu.ip]).chars().nth(1).unwrap();
                let hex3 = format!("{:02X}", rom[self.cpu.ip + 1])
                    .chars()
                    .nth(0)
                    .unwrap();
                let hex4 = format!("{:02X}", rom[self.cpu.ip + 1])
                    .chars()
                    .nth(1)
                    .unwrap();

                match (hex1, hex2, hex3, hex4) {
                    ('0', '0', 'E', '0') => {
                        screen_bits = [false; 64 * 32];
                        println!("[CLRS]")
                    }
                    ('0', '0', 'E', 'E') => {
                        println!("[RET]");
                        self.sp -= 1;
                        self.cpu.ip = self.stack[self.sp] as usize;
                    }
                    ('1', n1, n2, n3) => {
                        self.cpu.ip =
                            usize::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16)
                                .unwrap()
                                - 512;
                        // The game code should be in memory, so we subtract its offset
                        // Maybe will have to move the game code into memory in the future
                        println!("[JUMP]: {:?}", self.cpu.ip);
                        continue; // does not increment counter
                    }
                    ('2', n1, n2, n3) => {
                        self.stack[self.sp] = self.cpu.ip as u16;
                        self.sp += 1;
                        self.cpu.ip =
                            usize::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16)
                                .unwrap()
                                - 512;
                        // * divided by two because a instruction is two bytes?
                        println!("[CALL] {}", self.cpu.ip);
                        continue; // does not increment counter
                    }
                    ('3', r, n1, n2) => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        println!(
                            "[IIE ]: V{r} {} == {}",
                            self.cpu.regs[reg_i], self.cpu.regs[reg_i]
                        ); // Ignore if not equal
                        let value_to_check =
                            u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                        if self.cpu.regs[reg_i] == value_to_check {
                            self.cpu.ip += 2;
                            println!("        [INFO]: Ignored.");
                        }
                    }
                    ('4', r, n1, n2) => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        let value_to_check =
                            u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                        println!(
                            "[IINE]: V{} {} != {}",
                            reg_i, self.cpu.regs[reg_i], value_to_check
                        ); // Ignore if equal
                        if self.cpu.regs[reg_i] != value_to_check {
                            self.cpu.ip += 2;
                            println!("  [INFO]: Ignored.");
                        }
                    }
                    ('6', r, n1, n2) => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        let value_to_set =
                            u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                        self.cpu.regs[reg_i] = value_to_set;
                        println!("[MSET]: {} at V{}", self.cpu.regs[reg_i], reg_i);
                    }
                    ('7', r, n1, n2) => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        let value_to_add =
                            u8::from_str_radix(format!("{}{}", n1, n2).as_str(), 16).unwrap();
                        self.cpu.regs[reg_i] = self.cpu.regs[reg_i].wrapping_add(value_to_add);
                        println!("[INCR]: V{reg_i} += {value_to_add}");
                    }
                    ('8', r1, r2, '0') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                        println!("[CPXY]: V{r1} = V{r2} = {}", self.cpu.regs[reg_i2]);
                        self.cpu.regs[reg_i1] = self.cpu.regs[reg_i2];
                    }
                    ('8', r1, r2, '1') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                        println!(
                            "[ORR ]: V{r1} | V{r2} = {:08b} | {:08b} = {:08b}",
                            self.cpu.regs[reg_i1],
                            self.cpu.regs[reg_i2],
                            self.cpu.regs[reg_i1] | self.cpu.regs[reg_i2]
                        );
                        self.cpu.regs[reg_i1] = self.cpu.regs[reg_i2] | self.cpu.regs[reg_i1];
                    }
                    ('8', r1, r2, '2') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                        println!(
                            "[ANDR]: V{r1} = V{r1} & V{r2} = {:08b} & {:08b} = {:08b}",
                            self.cpu.regs[reg_i1],
                            self.cpu.regs[reg_i2],
                            self.cpu.regs[reg_i1] & self.cpu.regs[reg_i2]
                        );
                        self.cpu.regs[reg_i1] = self.cpu.regs[reg_i1] & self.cpu.regs[reg_i2];
                    }
                    ('8', r1, r2, '3') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                        println!(
                            "[XORR]: V{r1} = V{r1} ^ V{r2} = {:08b} ^ {:08b} = {:08b}",
                            self.cpu.regs[reg_i1],
                            self.cpu.regs[reg_i2],
                            self.cpu.regs[reg_i1] ^ self.cpu.regs[reg_i2]
                        );
                        self.cpu.regs[reg_i1] = self.cpu.regs[reg_i1] ^ self.cpu.regs[reg_i2];
                    }
                    ('8', r1, r2, '4') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();

                        let sum = self.cpu.regs[reg_i1].overflowing_add(self.cpu.regs[reg_i2]);
                        self.cpu.regs[reg_i1] = sum.0;
                        self.cpu.regs[15] = match sum.1 {
                            true => 1,
                            false => 0,
                        };
                        println!("[ADDC]");
                    }
                    ('8', r1, r2, '5') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();

                        let sub = self.cpu.regs[reg_i1].overflowing_sub(self.cpu.regs[reg_i2]);
                        self.cpu.regs[reg_i1] = sub.0;
                        self.cpu.regs[15] = match sub.1 {
                            true => 0,
                            false => 1,
                        };
                        println!("[SUBC]"); // sub wrap carry if not borrow
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
                    ('9', r1, r2, '0') => {
                        let reg_i1 = usize::from_str_radix(r1.to_string().as_str(), 16).unwrap();
                        let reg_i2 = usize::from_str_radix(r2.to_string().as_str(), 16).unwrap();
                        println!("[SINE]: {reg_i1} != {reg_i2}");
                        if self.cpu.regs[reg_i1] != self.cpu.regs[reg_i2] {
                            println!("      [INFO]: Ignored.");
                            self.cpu.ip += 2;
                        }
                    }
                    ('A', n1, n2, n3) => {
                        self.cpu.mem_address =
                            u16::from_str_radix(format!("{}{}{}", n1, n2, n3).as_str(), 16)
                                .unwrap();
                        println!("[MPST]: {}", self.cpu.mem_address);
                    }
                    ('C', r, k1, k2) => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        let mut rng = rand::thread_rng();
                        let value_to_and =
                            u8::from_str_radix(format!("{}{}", k1, k2).as_str(), 16).unwrap();
                        let n: u8 = rng.gen::<u8>() & value_to_and;
                        self.cpu.regs[reg_i] = n;
                        println!("[RAND]: {r}");
                    }
                    ('D', x, y, n) => {
                        let height = usize::from_str_radix(n.to_string().as_str(), 16).unwrap();
                        let reg_x = u8::from_str_radix(x.to_string().as_str(), 16).unwrap();
                        let reg_y = u8::from_str_radix(y.to_string().as_str(), 16).unwrap();
                        let pos_x = self.cpu.regs[reg_x as usize] as usize;
                        let pos_y = self.cpu.regs[reg_y as usize] as usize;

                        self.cpu.regs[15] = 0;

                        // TODO: Handle screen wraping
                        for mut y in pos_y..pos_y + height {
                            let cur_line = self.memory[self.cpu.mem_address as usize + y - pos_y];
                            // trying my best to not do the most convoluted mess ever written
                            for mut x in pos_x..pos_x + 8 {
                                let mut local_x = x - pos_x;
                                local_x = 7 - local_x;

                                let sprite_pixel = ((cur_line >> local_x) & 1) != 0;
                                x = x % 64;
                                y = y % 32;
                                let cur_screen_pixel = screen_bits[x + y * 64];
                                let new_pixel = (sprite_pixel || cur_screen_pixel)
                                    && !(sprite_pixel && cur_screen_pixel);
                                screen_bits[x + y * 64] = new_pixel;
                                if cur_screen_pixel == true && new_pixel == false {
                                    self.cpu.regs[15] = 1; // collision acontecey
                                }
                            }
                        }

                        println!("[DSA]");
                    }
                    ('E', r, '9', 'E') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        println!("[SIKP]: Checking {}", self.cpu.regs[reg_i]);
                        if cur_pressed_keys[self.cpu.regs[reg_i] as usize] {
                            self.cpu.ip += 2;
                            println!("        [INFO]: Skipped")
                        }
                    }
                    ('E', r, 'A', '1') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        println!("[SINP]: Checking {}", self.cpu.regs[reg_i]); // skip if not pressed
                        if !cur_pressed_keys[self.cpu.regs[reg_i] as usize] {
                            self.cpu.ip += 2;
                            println!("        [INFO]: Skipped")
                        }
                    }
                    ('F', r, '0', '7') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        self.cpu.regs[reg_i] = self.dt as u8;
                        println!("[SRDT]: V{reg_i} = DT = {}", self.dt);
                    }
                    ('F', r, '0', 'A') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        println!("[WFI ]: V{reg_i}");
                        let mut was_pressed = false;
                        for (value, pressed) in cur_pressed_keys.iter().enumerate() {
                            if *pressed {
                                self.cpu.regs[reg_i] = value as u8;
                                was_pressed = true;
                            }
                        }
                        if !was_pressed {
                            continue;
                        }
                    }
                    ('F', r, '1', '5') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        self.dt = self.cpu.regs[reg_i] as usize;
                        println!("[SDTR]: DT = V{reg_i} = {}", self.dt);
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
                    ('F', r, '2', '9') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        let n = self.cpu.regs[reg_i];
                        self.cpu.mem_address = n as u16 * 5;
                        println!("[SITD]: {n} new address = {}", self.cpu.mem_address);
                    }
                    ('F', r, '3', '3') => {
                        let reg_i = usize::from_str_radix(r.to_string().as_str(), 16).unwrap();
                        let mem_address = self.cpu.mem_address as usize;
                        let to_text = format!("{:03}", self.cpu.regs[reg_i]);
                        self.memory[mem_address] =
                            to_text.chars().nth(0).unwrap().to_digit(10).unwrap() as u8;
                        self.memory[mem_address + 1] =
                            to_text.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                        self.memory[mem_address + 2] =
                            to_text.chars().nth(2).unwrap().to_digit(10).unwrap() as u8;
                        println!("[DBCD]: V{reg_i} to {}", self.cpu.mem_address);
                        println!(
                            "  [INFO]: Value = {}, in memory = {} {} {}",
                            self.cpu.regs[reg_i],
                            self.memory[mem_address],
                            self.memory[mem_address + 1],
                            self.memory[mem_address + 2]
                        );
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
                            self.cpu.regs[i] =
                                self.memory[(self.cpu.mem_address + i as u16) as usize];
                        }
                        println!("[RRD ]: To V{reg_i} at {}", self.cpu.mem_address);
                        // ! WASN'T WORKING CORRECTLY BECAUSE THE EMULATOR DIDN'T DUMP THE ROM INTO THE RAM, SO IT JUST COPIED A BUNCH OF ZEROS
                    }
                    _ => todo!("HEX: {}{}{}{}\nIP: {}", hex1, hex2, hex3, hex4, self.cpu.ip),
                }
                self.cpu.ip += 2;
            }
            while frame_start.elapsed().as_millis() < 1000 / 60 {}
        }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: [0; 16],
            ip: 0,
            mem_address: 0,
        }
    }
}

impl Chip8Emulator {
    pub fn new() -> Chip8Emulator {
        let mut memory = [0; 4096];
        memory[0..80].copy_from_slice(&[
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
        ]);

        Chip8Emulator {
            cpu: Cpu::new(),
            memory,
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
        }
    }
}
