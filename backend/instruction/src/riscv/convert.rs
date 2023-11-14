#![allow(unused)]

use llvm::llvmop::*;
use utils::SysycError::{self, RiscvGenError};

use crate::{
	riscv::{reg::*, riscvinstr::*, riscvop::*, value::*},
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
		instrs.push(ILoadInstr::new(Li, rd, Int(num)));
	} else {
		instrs.push(ILoadInstr::new(Lui, rd, Int(num >> 12)));
		instrs.push(ITriInstr::new(Addi, rd, rd, Int(num & 0xFFF)));
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

fn get_arith(
	rd: RiscvTemp,
	op: ArithOp,
	lhs: &Value,
	rhs: &Value,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) {
	match end_num(lhs) {
		Some(num) if is_commutative(&op) => {
			let rhs = into_reg(lhs, instrs, mgr);
			instrs.push(ITriInstr::new(to_iop(&op), rd, rhs, num.into()));
		}
		_ => {
			if let Some(num) = end_num(rhs) {
				let lhs = into_reg(lhs, instrs, mgr);
				instrs.push(ITriInstr::new(to_iop(&op), rd, lhs, num.into()));
			} else {
				let lhs = into_reg(lhs, instrs, mgr);
				let rhs = into_reg(rhs, instrs, mgr);
				instrs.push(RTriInstr::new(to_rop(&op), rd, lhs, rhs));
			}
		}
	}
}

pub fn riscv_arith(
	instr: &llvm::llvminstr::ArithInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let (lhs, rhs) = (&instr.lhs, &instr.rhs);
	let target = mgr.get(&instr.target);
	get_arith(target, instr.op, lhs, rhs, &mut instrs, mgr);
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_label(
	instr: &llvm::llvminstr::LabelInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	Ok(InstrSet::RiscvInstrSet(vec![LabelInstr::new(
		instr.label.clone(),
	)]))
}

fn get_slt(
	rd: RiscvTemp,
	lhs: &Value,
	rhs: &Value,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) {
	let lhs = into_reg(lhs, instrs, mgr);
	if let Some(num) = end_num(rhs) {
		instrs.push(ITriInstr::new(Slti, rd, lhs, num.into()));
	} else {
		let rhs = into_reg(rhs, instrs, mgr);
		instrs.push(RTriInstr::new(Slt, rd, lhs, rhs));
	}
}

pub fn riscv_comp(
	instr: &llvm::llvminstr::CompInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let (lhs, rhs) = (&instr.lhs, &instr.rhs);
	let target = mgr.get(&instr.target);
	match &instr.op {
		CompOp::EQ | CompOp::OEQ => {
			let tmp = mgr.new_temp();
			get_arith(tmp, ArithOp::Xor, lhs, rhs, &mut instrs, mgr);
			instrs.push(ITriInstr::new(Sltiu, target, tmp, 1.into()));
		}
		CompOp::NE | CompOp::ONE => {
			let tmp = mgr.new_temp();
			get_arith(tmp, ArithOp::Xor, lhs, rhs, &mut instrs, mgr);
			instrs.push(RTriInstr::new(Sltu, target, X0.into(), tmp));
		}
		CompOp::SLT | CompOp::OLT => {
			get_slt(target, lhs, rhs, &mut instrs, mgr);
		}
		CompOp::SLE | CompOp::OLE => {
			get_slt(target, rhs, lhs, &mut instrs, mgr);
			instrs.push(ITriInstr::new(Xori, target, target, 1.into()));
		}
		CompOp::SGT | CompOp::OGT => {
			get_slt(target, rhs, lhs, &mut instrs, mgr);
		}
		CompOp::SGE | CompOp::OGE => {
			get_slt(target, lhs, rhs, &mut instrs, mgr);
			instrs.push(ITriInstr::new(Xori, target, target, 1.into()));
		}
	}
	Ok(InstrSet::RiscvInstrSet(instrs))
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
