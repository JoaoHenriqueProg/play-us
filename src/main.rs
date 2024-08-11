use std::process::exit;
use std::fs;
use std::env;
use emulator::Emulator;

mod video;
mod emulator;
mod chip8;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("ROM path must be provided.");
        return
    }
    let rom_path = args[1].as_str();
    let rom = fs::read(rom_path);
    let rom = match rom {
        Ok(bytes) => bytes, 
        Err(_) => {
            println!("ROM path is invalid.");
            exit(-1);
        }
    };
    let mut emulator = chip8::Chip8Emulator::new();
    emulator.run(&rom);
}
