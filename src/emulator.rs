pub trait Emulator {
    fn run(&mut self, rom: &[u8]);
}