use super::memory::Memory;

pub const N_FLAG: u8 = 0b1000_0000;
pub const V_FLAG: u8 = 0b0100_0000;
// const B_FLAG: u8 = 0b0001_0000;
// const D_FLAG: u8 = 0b0000_1000;
// const I_FLAG: u8 = 0b0000_0100;
pub const Z_FLAG: u8 = 0b0000_0010;
pub const C_FLAG: u8 = 0b0000_0001;
pub const SIGN_BIT: u8 = 0b1000_0000;

pub enum Mode {
    Immediate,
    Implied,
}

pub enum Reg {
    A,
    X,
    Y,
    S,
}

#[derive(Copy, Clone)]
pub struct State {
    pub a: u8,
    pub pc: usize,
    pub x: u8,
    pub y: u8,
    pub status: u8,
}

pub struct Processor {
    pub mem: Memory,
    pub state: State,
    pub cycles: u32,
}

impl Processor {
    pub fn get_pc(&self) -> usize {
        self.state.pc
    }

    pub fn update_pc(&mut self, delta: i32) -> &mut Self {
        println!("Update pc {} with {}", self.state.pc, delta);
        if delta.is_negative() {
            self.state.pc -= delta.wrapping_abs() as u32 as usize;
        } else {
            self.state.pc += delta as usize;
        }
        self
    }

    pub fn get_reg(&self, reg: Reg) -> u8 {
        match reg {
            Reg::X => self.state.x,
            Reg::Y => self.state.y,
            Reg::A => self.state.a,
            Reg::S => self.state.status,
        }
    }

    pub fn set_reg(&mut self, reg: Reg, value: u8) -> &mut Self {
        match reg {
            Reg::X => self.state.x = value,
            Reg::Y => self.state.y = value,
            Reg::A => self.state.a = value,
            Reg::S => self.state.status = value,
        };
        self
    }

    pub fn update_cycles(&mut self, cycles: u32) -> &mut Self {
        self.cycles += cycles;
        self
    }

    /**
     * Calculate new Status flag based on the operation
     */
    pub fn update_status(
        &mut self,
        m: u8,
        n: u8,
        result: u8,
        flags: u8,
    ) -> &mut Self {
        let mut new_status = self.get_reg(Reg::S);
        let mut merge_status = |flag: u8, value: bool| {
            if value {
                new_status |= flag
            } else {
                new_status &= !flag
            }
        };

        if flags & C_FLAG != 0 {
            merge_status(C_FLAG, m as u16 + n as u16 > 0xFF);
        }

        if flags & Z_FLAG != 0 {
            merge_status(Z_FLAG, result == 0);
        }

        if flags & N_FLAG != 0 {
            merge_status(N_FLAG, result & SIGN_BIT != 0);
        }

        // Overflow logic is a bit more complicated

        // XOR-ing m & n is going to clear the SIGN_BIT if it's not == in BOTH
        if flags & V_FLAG != 0 {
            let operands_match = ((m ^ n) & SIGN_BIT) == 0;
            let result_operands_match = ((m ^ result) & SIGN_BIT) == 0;
            let overflow = operands_match && !result_operands_match;

            merge_status(V_FLAG, overflow);
        }

        self.set_reg(Reg::S, new_status);

        self
    }
}