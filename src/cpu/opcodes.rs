use super::base::{Mode, Processor, Reg, C_FLAG, N_FLAG, V_FLAG, Z_FLAG};

impl Processor {
    pub fn adc(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let accumulator = self.state.a;
        let carry = self.state.status & 1;
        println!(
            "operand {} accumulator {} carry {}",
            operand, accumulator, carry
        );
        let (mut result, ..) = accumulator.overflowing_add(operand);
        result += carry;
        self.set_reg(Reg::A, result)
            .update_pc(2)
            .update_status(
                accumulator,
                operand,
                result,
                N_FLAG | Z_FLAG | C_FLAG | V_FLAG,
            )
            .update_cycles(2);
    }

    pub fn and(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let accumulator = self.get_reg(Reg::A);
        let result = accumulator & operand;
        self.set_reg(Reg::A, result)
            .update_pc(2)
            .update_status(accumulator, operand, result, N_FLAG | Z_FLAG)
            .update_cycles(2);
    }
    pub fn asl(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let result = operand << 1;
        let accumulator = self.get_reg(Reg::A);
        self.set_reg(Reg::A, result)
            .update_status(
                accumulator,
                operand,
                result,
                Z_FLAG | C_FLAG | N_FLAG,
            )
            .update_cycles(2);
    }

    pub fn lda(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        println!("Load accumulator with {}", operand);
        self.set_reg(Reg::A, operand)
            .update_pc(2)
            .update_status(operand, operand, operand, Z_FLAG | N_FLAG)
            .update_cycles(2);
    }

    pub fn sec(&mut self, _mode: Mode) {
        println!("Set carry flag");
        self.state.status |= C_FLAG;
        self.update_pc(1).update_cycles(2);
    }

    pub fn nop(&mut self, _mode: Mode) {
        println!("NOP");
        self.update_pc(1).update_cycles(1);
    }
}
