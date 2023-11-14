#![allow(unused)]

use llvm::llvmop::*;
use utils::SysycError::{self, RiscvGenError, SystemError};

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
		instrs.push(IBinInstr::new(Li, rd, num.into()));
	} else {
		instrs.push(IBinInstr::new(Lui, rd, (num >> 12).into()));
		instrs.push(ITriInstr::new(Addi, rd, rd, (num & 0xFFF).into()));
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
	if can_to_iop(&op) {
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
	} else {
		let lhs = into_reg(lhs, instrs, mgr);
		let rhs = into_reg(rhs, instrs, mgr);
		instrs.push(RTriInstr::new(to_rop(&op), rd, lhs, rhs));
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
	let mut instrs: RiscvInstrSet = Vec::new();
	let target = mgr.get(&instr.target);
	let from = &instr.lhs;
	if from.is_num() {
		Err(RiscvGenError("don't convert immediate number".to_owned()))?
	} else {
		let from = into_reg(from, &mut instrs, mgr);
		let op = match instr.op {
			ConvertOp::Float2Int => Float2Int,
			ConvertOp::Int2Float => Int2Float,
		};
		instrs.push(RBinInstr::new(op, target, from));
	}
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_jump(
	instr: &llvm::llvminstr::JumpInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let to = instr.target.clone().into();
	instrs.push(BranInstr::new(BEQ, X0.into(), X0.into(), to));
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_cond(
	instr: &llvm::llvminstr::JumpCondInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let cond = into_reg(&instr.cond, &mut instrs, mgr);
	let to_true = instr.target_true.clone().into();
	let to_false = instr.target_false.clone().into();
	instrs.push(BranInstr::new(BNE, cond, X0.into(), to_true));
	instrs.push(BranInstr::new(BEQ, X0.into(), X0.into(), to_false));
	Ok(InstrSet::RiscvInstrSet(instrs))
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
	let mut instrs: RiscvInstrSet = Vec::new();
	if let Some(val) = &instr.value {
		if let Some(num) = end_num(val) {
			instrs.push(IBinInstr::new(Li, A0.into(), num.into()));
		} else {
			let tmp = into_reg(val, &mut instrs, mgr);
			instrs.push(RTriInstr::new(Add, A0.into(), X0.into(), tmp));
		}
	}
	instrs.push(NoArgInstr::new(Ret));
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_alloc(
	instr: &llvm::llvminstr::AllocInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let size = &instr.length;
	if let Some(num) = end_num(size) {
		let num = (num + 15) & -16;
		instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-num).into()));
	} else {
		Err(RiscvGenError("array size should be constant".to_owned()))?
		// let num = into_reg(size, &mut instrs, mgr);
		// instrs.push(RTriInstr::new(Sub, SP.into(), SP.into(), num));
	}
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_store(
	instr: &llvm::llvminstr::StoreInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let addr = into_reg(&instr.addr, &mut instrs, mgr);
	let value = into_reg(&instr.value, &mut instrs, mgr);
	instrs.push(IBinInstr::new(SW, value, (0, addr).into()));
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_load(
	instr: &llvm::llvminstr::LoadInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let addr = into_reg(&instr.addr, &mut instrs, mgr);
	let rd = mgr.get(&instr.target);
	instrs.push(IBinInstr::new(LWU, rd, (0, addr).into()));
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_gep(
	instr: &llvm::llvminstr::GEPInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let rd = mgr.get(&instr.target);
	let (lhs, rhs) = (&instr.addr, &instr.offset);
	get_arith(rd, llvm::llvmop::ArithOp::Add, lhs, rhs, &mut instrs, mgr);
	Ok(InstrSet::RiscvInstrSet(instrs))
}

pub fn riscv_call(
	instr: &llvm::llvminstr::CallInstr,
	mgr: &mut TempManager,
) -> Result<InstrSet, SysycError> {
	todo!()
}
