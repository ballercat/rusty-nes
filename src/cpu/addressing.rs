use super::base::Processor;

pub enum Mode {
    Immediate,
    Implied,
}

impl Processor {
    pub fn lookup(&mut self, mode: Mode) -> u8 {
        match mode {
            Mode::Immediate => {
                println!("Lookup Immediate value at {}", self.state.pc + 1);
                self.mem.read(self.state.pc + 1)
            }
            Mode::Implied => {
                self.cycles += 1;
                0
            }
        }
    }
}
