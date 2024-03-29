use llvm::{ArithOp, CompKind, CompOp, VarType};
use value::{BType, BinaryOp, FuncRetType};

pub fn to_arith(op: BinaryOp, type_t: VarType) -> ArithOp {
	match (op, type_t) {
		(BinaryOp::Add, VarType::I32) => ArithOp::Add,
		(BinaryOp::Add, VarType::F32) => ArithOp::Fadd,
		(BinaryOp::Sub, VarType::I32) => ArithOp::Sub,
		(BinaryOp::Sub, VarType::F32) => ArithOp::Fsub,
		(BinaryOp::Mul, VarType::I32) => ArithOp::Mul,
		(BinaryOp::Mul, VarType::F32) => ArithOp::Fmul,
		(BinaryOp::Div, VarType::I32) => ArithOp::Div,
		(BinaryOp::Div, VarType::F32) => ArithOp::Fdiv,
		(BinaryOp::Mod, VarType::I32) => ArithOp::Rem,
		_ => unreachable!(),
	}
}

pub fn to_comp(op: BinaryOp, type_t: VarType) -> CompOp {
	match (op, type_t) {
		(BinaryOp::LT, VarType::I32) => CompOp::SLT,
		(BinaryOp::LT, VarType::F32) => CompOp::OLT,
		(BinaryOp::LE, VarType::I32) => CompOp::SLE,
		(BinaryOp::LE, VarType::F32) => CompOp::OLE,
		(BinaryOp::GT, VarType::I32) => CompOp::SGT,
		(BinaryOp::GT, VarType::F32) => CompOp::OGT,
		(BinaryOp::GE, VarType::I32) => CompOp::SGE,
		(BinaryOp::GE, VarType::F32) => CompOp::OGE,
		(BinaryOp::NE, VarType::I32) => CompOp::NE,
		(BinaryOp::NE, VarType::F32) => CompOp::ONE,
		(BinaryOp::EQ, VarType::I32) => CompOp::EQ,
		(BinaryOp::EQ, VarType::F32) => CompOp::OEQ,
		_ => unreachable!(),
	}
}

pub fn get_comp_kind(var_type: VarType) -> CompKind {
	match var_type {
		VarType::I32 => CompKind::Icmp,
		VarType::F32 => CompKind::Fcmp,
		_ => unreachable!(),
	}
}

pub fn type_convert(from: &value::VarType) -> VarType {
	match (from.type_t, from.dims.is_empty()) {
		(BType::Int, true) => VarType::I32,
		(BType::Float, true) => VarType::F32,
		(BType::Int, false) => VarType::I32Ptr,
		(BType::Float, false) => VarType::F32Ptr,
	}
}

pub fn func_type_convert(from: &value::FuncRetType) -> VarType {
	match from {
		FuncRetType::Int => VarType::I32,
		FuncRetType::Float => VarType::F32,
		FuncRetType::Void => VarType::Void,
	}
}
