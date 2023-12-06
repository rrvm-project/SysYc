use instruction::temp::Temp;
use rrvm::program::RiscvFunc;

pub fn spill(_func: &mut RiscvFunc, _node: Temp) {
	todo!("Alloctor can't spill register now")
}
