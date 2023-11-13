#![allow(unused)]

use llvm::llvmop::*;
use utils::SysycError::{self, RiscvGenError};

use crate::{
	riscv::{reg::RiscvReg, riscvinstr::*, riscvop::*, value::*},
	temp::TempManager,
	InstrSet, RiscvInstrSet,
};

fn i32_to_reg(
	num: i32,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> RiscvTemp {
	let rd = mgr.new_temp();
	if is_lower(num) {
		instrs.push(ILoadInstr::new(BiLoadImmOp::Li, rd, Int(num)));
	} else {
		instrs.push(ILoadInstr::new(BiLoadImmOp::Lui, rd, Int(num >> 12)));
		instrs.push(ITriInstr::new(ITriInstrOp::Addi, rd, rd, Int(num & 0xFFF)));
	}
	rd
}

fn f32_to_reg(
	num: f32,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> RiscvTemp {
	i32_to_reg(num.to_bits() as i32, instrs, mgr)
}

fn end_num(val: &llvm::llvmop::Value) -> Option<i32> {
	match val {
		Value::Int(v) => is_lower(*v).then_some(*v),
		_ => None,
	}
}

fn into_reg(
	val: &llvm::llvmop::Value,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> RiscvTemp {
	match val {
		Value::Int(num) => i32_to_reg(*num, instrs, mgr),
		Value::Float(num) => f32_to_reg(*num, instrs, mgr),
		Value::Temp(temp) => mgr.get(temp),
	}
}

pub fn riscv_arith(
	instr: &llvm::llvminstr::ArithInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let (lhs, rhs) = if instr.lhs.is_num() {
		(&instr.rhs, &instr.lhs)
	} else {
		(&instr.lhs, &instr.rhs)
	};
	let lhs = into_reg(lhs, &mut instrs, mgr);
	if let Some(num) = end_num(&instr.rhs) {
		let op = match &instr.op {
			ArithOp::Add => ITriInstrOp::Addi,
			ArithOp::Sub => ITriInstrOp::Subi,
			ArithOp::Mul => ITriInstrOp::Muli,
			ArithOp::Div => ITriInstrOp::Divi,
			ArithOp::Rem => ITriInstrOp::Remi,
			ArithOp::And => ITriInstrOp::Andi,
			ArithOp::Or => ITriInstrOp::Ori,
			ArithOp::Xor => ITriInstrOp::Xori,
			ArithOp::Shl => ITriInstrOp::Slli,
			ArithOp::Lshr => ITriInstrOp::Srli,
			ArithOp::Ashr => ITriInstrOp::Srai,
			_ => Err(RiscvGenError("use float op with integer".to_string()))?,
		};
		let rd = mgr.new_temp();
		instrs.push(ITriInstr::new(op, rd, lhs, RiscvImm::Int(num)));
	} else {
		let op = match &instr.op {
			ArithOp::Add => RTriInstrOp::Add,
			ArithOp::Sub => RTriInstrOp::Sub,
			ArithOp::Mul => RTriInstrOp::Mul,
			ArithOp::Div => RTriInstrOp::Div,
			ArithOp::Rem => RTriInstrOp::Rem,
			ArithOp::And => RTriInstrOp::And,
			ArithOp::Or => RTriInstrOp::Or,
			ArithOp::Xor => RTriInstrOp::Xor,
			ArithOp::Shl => RTriInstrOp::Sll,
			ArithOp::Lshr => RTriInstrOp::Srl,
			ArithOp::Ashr => RTriInstrOp::Sra,
			ArithOp::Fadd => RTriInstrOp::Fadd,
			ArithOp::Fsub => RTriInstrOp::Fsub,
			ArithOp::Fmul => RTriInstrOp::Fmul,
			ArithOp::Fdiv => RTriInstrOp::Fdiv,
		};
		let rhs = into_reg(rhs, &mut instrs, mgr);
		let rd = mgr.new_temp();
		instrs.push(RTriInstr::new(op, rd, lhs, rhs));
	}
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_label(
	instr: &llvm::llvminstr::LabelInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_comp(
	instr: &llvm::llvminstr::CompInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_convert(
	instr: &llvm::llvminstr::ConvertInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_jump(
	instr: &llvm::llvminstr::JumpInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_cond(
	instr: &llvm::llvminstr::JumpCondInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_phi(
	instr: &llvm::llvminstr::PhiInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	unreachable!("phi instruction should be solved in mid end")
}

pub fn riscv_ret(
	instr: &llvm::llvminstr::RetInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_alloc(
	instr: &llvm::llvminstr::AllocInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_store(
	instr: &llvm::llvminstr::StoreInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_load(
	instr: &llvm::llvminstr::LoadInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_gep(
	instr: &llvm::llvminstr::GEPInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}

pub fn riscv_call(
	instr: &llvm::llvminstr::CallInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}
