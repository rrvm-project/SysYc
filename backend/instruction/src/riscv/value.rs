use super::reg::RiscvReg;
use llvm::temp::Temp;

pub enum Value {
	Imm(i32),
	Temp(Temp),
	Reg(RiscvReg),
}
