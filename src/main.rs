use emulator::Emulator;

mod emulator;
mod chip8;

fn main() {
    let rom = include_bytes!("..\\roms\\Space Invaders [David Winter].ch8");
    let mut emulator = chip8::Chip8Emulator::new();
    emulator.run(rom);
}
