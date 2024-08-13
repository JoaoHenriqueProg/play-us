use emulator::Emulator;
use std::env;
use std::fs;
use std::process::exit;

mod chip8;
mod emulator;
mod video;

#[derive(Clone, Copy)]
enum Emulators {
    Chip8,
}

impl ToString for Emulators {
    fn to_string(&self) -> String {
        match self {
            Emulators::Chip8 => "Chip8 emulator".to_string(),
        }
    }
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
    let rom_type = rom_path.split(".").last().unwrap();
    match (rom_type, emulator_to_use) {
        ("ch8", Emulators::Chip8) => {}
        _ => {
            println!(
                ".{rom_type} is not a valid extension for {} to execute",
                emulator_to_use.to_string()
            );
            return;
        }
    }
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
