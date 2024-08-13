use emulator::Emulator;
use std::env;
use std::fs;
use std::process::exit;

mod chip8;
mod emulator;
mod video;

enum Emulators {
    Chip8,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let emulator_to_use;
    match args.get(1) {
        Some(arg1) => {
            // I have no idea how to match on a string
            if arg1.eq("help") {
                println!("emulators avaiblable:");
                println!("  chip8 - Chip8 emulator");
                return;
            } else if arg1.eq("chip8") {
                emulator_to_use = Emulators::Chip8;
            } else {
                println!("rom_path was not provided!");
                println!("usage: emulator rom_path");
                println!("\"help\" for more information, such as emulators available");
                return;
            }
        }
        None => {
            println!("usage: emulator rom_path");
            println!("\"help\" for more information, such as emulators available");
            return;
        }
    }
    let rom_path = args[2].as_str();
    let rom = fs::read(rom_path);
    let rom = match rom {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("ROM path is invalid.");
            exit(-1);
        }
    };

    // I foresee this will not work once I implement the next emulator
    // Seems like a future me problem
    let mut emulator = match emulator_to_use {
        Emulators::Chip8 => chip8::Chip8Emulator::new(),
    };
    emulator.run(&rom);
}
