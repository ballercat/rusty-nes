use super::memory::{Memory, ZERO_PAGE_TOP};

pub const N_FLAG: u8 = 0b1000_0000;
pub const V_FLAG: u8 = 0b0100_0000;
pub const B_FLAG: u8 = 0b0001_0000; // See PHP/BRK etc
pub const F_FLAG: u8 = 0b0010_0000; // f for five not a real flag
pub const D_FLAG: u8 = 0b0000_1000;
pub const I_FLAG: u8 = 0b0000_0100;
pub const Z_FLAG: u8 = 0b0000_0010;
pub const C_FLAG: u8 = 0b0000_0001;
pub const SIGN_BIT: u8 = 0b1000_0000;

pub enum Reg {
    A,
    X,
    Y,
    S,
    SP,
}

#[derive(Copy, Clone, Debug)]
pub struct State {
    pub a: u8,
    pub sp: u8,
    pub pc: usize,
    pub x: u8,
    pub y: u8,
    pub status: u8,
}

#[derive(Debug)]
pub struct Processor {
    pub mem: Memory,
    pub state: State,
    pub cycles: u32,
}

impl Processor {
    pub fn new(mem: Option<Memory>) -> Processor {
        let state = State {
            a: 0,
            sp: 0,
            pc: 0,
            x: 0,
            y: 0,
            status: 0,
        };
        Processor {
            mem: mem.unwrap_or(Memory::new()),
            state,
            cycles: 0,
        }
    }
    pub fn get_pc(&self) -> usize {
        self.state.pc
    }

    pub fn stack_top(&self) -> usize {
        ZERO_PAGE_TOP + self.state.sp as usize
    }

    pub fn stack_push(&mut self, value: u8) {
        self.mem.write(self.stack_top(), value);
        self.state.sp = if self.state.sp == 0 {
            0xff
        } else {
            self.state.sp - 1
        };
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.state.sp = if self.state.sp == 0xff {
            0
        } else {
            self.state.sp + 1
        };
        let result = self.mem.read(self.stack_top());
        result
    }

    pub fn update_pc(&mut self, delta: i32) -> &mut Self {
        if delta.is_negative() {
            self.state.pc -= delta.wrapping_abs() as u32 as usize;
        } else {
            self.state.pc += delta as usize;
        }
        self
    }

    pub fn jump(&mut self, new_pc: usize) -> &mut Self {
        self.state.pc = new_pc;
        self
    }

    pub fn get_reg(&self, reg: Reg) -> u8 {
        match reg {
            Reg::X => self.state.x,
            Reg::Y => self.state.y,
            Reg::A => self.state.a,
            Reg::S => self.state.status,
            Reg::SP => self.state.sp,
        }
    }

    pub fn set_reg(&mut self, reg: Reg, value: u8) -> &mut Self {
        match reg {
            Reg::X => self.state.x = value,
            Reg::Y => self.state.y = value,
            Reg::A => self.state.a = value,
            Reg::S => self.state.status = value,
            Reg::SP => self.state.sp = value,
        };
        self
    }

    pub fn update_cycles(&mut self, cycles: u32) -> &mut Self {
        self.cycles += cycles;
        self
    }

    pub fn update_n_flag(&mut self, value: u8) -> &mut Self {
        if value & SIGN_BIT != 0 {
            self.state.status |= N_FLAG;
        } else {
            self.state.status &= !N_FLAG;
        }
        self
    }

    pub fn update_z_flag(&mut self, value: u8) -> &mut Self {
        if value == 0 {
            self.state.status |= Z_FLAG;
        } else {
            self.state.status &= !Z_FLAG;
        }
        self
    }

    pub fn set_carry(&mut self, toggle: bool) -> &mut Self {
        if toggle {
            self.state.status |= C_FLAG;
        } else {
            self.state.status &= !C_FLAG;
        }
        self
    }

    pub fn set_z(&mut self, toggle: bool) -> &mut Self {
        if toggle {
            self.state.status |= Z_FLAG;
        } else {
            self.state.status &= !Z_FLAG;
        }
        self
    }

    pub fn set_n(&mut self, toggle: bool) -> &mut Self {
        if toggle {
            self.state.status |= N_FLAG;
        } else {
            self.state.status &= !N_FLAG;
        }
        self
    }

    pub fn update_v(&mut self, lhs: u8, rhs: u8, result: u8) -> &mut Self {
        if !(lhs ^ rhs) & (lhs ^ result) & SIGN_BIT != 0 {
            self.state.status |= V_FLAG;
        } else {
            self.state.status &= !V_FLAG;
        }
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

        if flags & C_FLAG != 0 {
            if  m as u16 + n as u16 > 0xFF {
                new_status |= C_FLAG;
            } else {
                new_status &= !C_FLAG;
            }
        }

        if flags & Z_FLAG != 0 {
            if result == 0 {
                new_status |= Z_FLAG;
            } else {
                new_status &= !Z_FLAG;
            }
        }

        if flags & N_FLAG != 0 {
            if result & SIGN_BIT > 0 {
                new_status |= SIGN_BIT;
            } else {
                new_status &= !SIGN_BIT;
            }
        }

        // XOR-ing m & n is going to clear the SIGN_BIT if it's not == in BOTH
        if flags & V_FLAG != 0 {
            if !(m ^ n) & (m ^ result) & SIGN_BIT != 0 {
                new_status |= V_FLAG;
            } else {
                new_status &= !V_FLAG;
            }
        }

        self.set_reg(Reg::S, new_status);

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_status_flags() {
        //  http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let overflow_table = [
            // "#0 No unsigned carry or signed overflow",
            (0x50, 0x10, 0x60, 0b0000_0000),
            // "#1 No unsigned carry but signed overflow",
            (0x50, 0x50, 0xae, 0b0100_0000),
            // "#2 No unsigned carry or signed overflow",
            (0x50, 0x90, 0xe0, 0b0000_0000),
            // "#3 Unsigned carry, but no signed overflow",
            (0x50, 0xd0, 0x120, 0b0000_0000),
            // "#4 No unsigned carry or signed overflow",
            (0xd0, 0x10, 0xe0, 0b0000_0000),
            // "#5 Unsigned carry but no signed overflow",
            (0xd0, 0x50, 0x120, 0b0000_0000),
            // "#6 Unsigned carry and signed overflow",
            (0xd0, 0x90, 0x160, 0b0100_0000),
            // "#7 Unsigned carry, but no signed overflow",
            (0xd0, 0xd0, 0x1a0, 0b0000_0000),
        ];

        let mut cpu = Processor::new(None);

        for i in 0..overflow_table.len() {
            let (m, n, result, expected) = overflow_table[i];
            cpu.update_status(m, n, result as u8, V_FLAG);
            assert_eq!(
                cpu.state.status, expected,
                "VFLAG m: {} n: {} result: {}",
                m, n, result
            );
        }
    }
}
