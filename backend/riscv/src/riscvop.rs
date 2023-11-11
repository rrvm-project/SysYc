use llvm::temp::Temp;
use serde_derive::Serialize;
use std::fmt::Display;

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Regs {
	X0, // always zero
	RA, // return address
	SP, // stack pointer
	GP, // global pointer
	TP, // thread pointer
	T0,
	T1,
	T2,
	FP, // frame pointer
	S1,
	A0,
	A1,
	A2,
	A3,
	A4,
	A5,
	A6,
	A7,
	S2,
	S3,
	S4,
	S5,
	S6,
	S7,
	S8,
	S9,
	S10,
	S11,
	T3,
	T4,
	T5,
	T6,
}
impl Display for Regs {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}
//可能有一个小问题 像是处理非全局的float的时候load是它的二进制表示换算成的整数
//reference:https://jborza.com/post/2021-05-09-floating-point-adventures/
pub enum Value {
	Imm(i32),//immediate
	Float(f32),
	Register(Regs),
	StackVal(i32),
    Temp(Temp)
}
impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Imm(v) => write!(f, "{}", v),
			Self::Float(v) => write!(f, "{}", v),
			Self::Register(v) => {
				f.write_str(&serde_json::to_string(v).unwrap().trim_matches('\"'))
			}
			Self::StackVal(v) => write!(f, "{}(sp)", v),
            Self::Temp(v)=> write!(f, "{}",v),
		}
	}
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArithOp {
	Add,
	Sub,
	Mul,
	Div,
	Rem,
	Sll,
	Srl,
	Sra,
	And,
	Or,
	Xor,
}

impl Display for ArithOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CmpOp {
	Slt,
	Sgt,
	Feq,
	Flt,
	Fle,
}
impl Display for CmpOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EqOp {
	Seqz,
	Snez,
}
impl Display for EqOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BranchOp {
    Beq,
    Bne,
    Blt,
    Bge
}
impl Display for BranchOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}
//这个可能优化要用
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BeqzOp{
    Beqz,
    Bnez,
}
impl Display for BeqzOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConvertOp {
    Fcvtsw,//convert from int
    Fcvtws,//convert to int
}
impl Display for ConvertOp {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(&serde_json::to_string(self).unwrap().trim_matches('\"'));
		Ok(())
	}
}