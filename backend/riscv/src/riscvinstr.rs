use crate::riscvop::*;
use llvm::{label::Label, llvmvar::VarType, temp::Temp};
use std::fmt::Display;
pub trait RiscvInstr: Display {
	fn get_write(&self) -> Value {
		Value::Register(Regs::X0)
	}
	fn get_read(&self) -> Vec<Value> {
		Vec::new()
	}
}
pub struct ArithInstr {
	pub tar: Value,
	pub op: ArithOp,
	pub lhs: Value,
	pub rhs: Value,
}
pub struct LabelInstr {
	pub label: Label,
}
pub struct CompInstr {
	pub tar: Value,
	pub op: CmpOp,
	pub lhs: Value,
	pub rhs: Value,
}
//看看brcond和jmp需不需要挂一个块id之类的，感觉没太有必要（
pub struct BrCondInstr {
	pub tar: Label,
	pub op: BranchOp,
	pub lhs: Value,
	pub rhs: Value,
}
pub struct JmpInstr {
	pub tar: Label,
}
pub struct BeqzInstr {
	pub tar: Label,
	pub op: BeqzOp,
	pub lhs: Value,
}
//return instr 就不做了 检查到了函数末尾的时候自动生成吧
//alloc搞个假的？
pub struct AllocInstr {
	pub length: usize,
}
pub struct StoreInstr {
	pub value: Value,
	pub offset: Value,
}
pub struct LoadInstr {
	pub target: Value,
	pub offset: Value,
}
pub struct CallInstr {
	pub func: Label,
	pub params: Vec<(Value, Value)>, //先看看能不能搞成一个映射的形式,(虚拟寄存器/立即数+物理寄存器/栈)
}
pub struct MvInstr{
    pub src: Value,
    pub dst: Value,
}
pub struct LiInstr{
    pub dst: Value,
    pub src: Value,
}
pub struct ConvertInstr{
    pub dst: Value,
    pub src: Value,
    pub op: ConvertOp,
}