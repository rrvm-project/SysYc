use std::cmp::max;

use llvm::llvmop::*;
use utils::{
	errors::{Result, SysycError::*},
	Label,
};

use crate::{
	riscv::{reg::*, riscvinstr::*, riscvop::*, value::*},
	temp::TempManager,
	RiscvInstrSet,
};

fn reg_cost(val: &llvm::llvmop::Value) -> i32 {
	fn i32_cost(num: i32) -> i32 {
		match num {
			0 => 0,
			_ => 1 + (!is_lower(num) as i32),
		}
	}
	match val {
		Value::Int(num) => i32_cost(*num),
		Value::Float(num) => i32_cost(num.to_bits() as i32),
		Value::Temp(_) => 0,
	}
}

pub fn i32_to_reg(
	num: i32,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> RiscvTemp {
	if num == 0 {
		return RiscvTemp::PhysReg(X0); // 代价不同、最小化代价
	}
	let rd = mgr.new_temp();
	if is_lower(num) {
		instrs.push(IBinInstr::new(Li, rd, num.into()));
	} else {
		let (high, low) = if (num & 0x800) != 0 {
			((num >> 12) + 1, num & 0xFFF | (-1 << 12))
		} else {
			(num >> 12, num & 0xFFF)
		};
		instrs.push(IBinInstr::new(Lui, rd, high.into()));
		instrs.push(ITriInstr::new(Addi, rd, rd, low.into()));
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
			Some(num) if is_commutative(&op) && reg_cost(lhs) > reg_cost(rhs) => {
				let rhs = into_reg(rhs, instrs, mgr);
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
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let (lhs, rhs) = (&instr.lhs, &instr.rhs);
	let target = mgr.get(&instr.target);
	get_arith(target, instr.op, lhs, rhs, &mut instrs, mgr);
	Ok(instrs)
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
) -> Result<RiscvInstrSet> {
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
	Ok(instrs)
}

pub fn riscv_convert(
	instr: &llvm::llvminstr::ConvertInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
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
	Ok(instrs)
}

pub fn riscv_jump(
	instr: &llvm::llvminstr::JumpInstr,
	_mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let to = instr.target.clone().into();
	instrs.push(BranInstr::new(Beq, X0.into(), X0.into(), to));
	Ok(instrs)
}

pub fn riscv_cond(
	instr: &llvm::llvminstr::JumpCondInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let cond = into_reg(&instr.cond, &mut instrs, mgr);
	let to_true = instr.target_true.clone().into();
	let to_false = instr.target_false.clone().into();
	instrs.push(BranInstr::new(Bne, cond, X0.into(), to_true));
	instrs.push(BranInstr::new(Beq, X0.into(), X0.into(), to_false));
	Ok(instrs)
}

pub fn riscv_phi(
	_instr: &llvm::llvminstr::PhiInstr,
	_mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	unreachable!("phi instruction should be solved before instruction selcetion")
}

pub fn riscv_ret(
	instr: &llvm::llvminstr::RetInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
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
	Ok(instrs)
}

pub fn riscv_alloc(
	instr: &llvm::llvminstr::AllocInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let size = &instr.length;
	if let Some(num) = end_num(size) {
		let num = (num + 15) & -16;
		let target = mgr.get(&instr.target);
		instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-num).into()));
		instrs.push(RTriInstr::new(Add, target, X0.into(), SP.into()));
	} else {
		let target = mgr.get(&instr.target);
		let num = into_reg(size, &mut instrs, mgr);
		instrs.push(RTriInstr::new(Sub, SP.into(), SP.into(), num));
		instrs.push(RTriInstr::new(Add, target, X0.into(), SP.into()));
	}
	Ok(instrs)
}

pub fn riscv_store(
	instr: &llvm::llvminstr::StoreInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let addr = into_reg(&instr.addr, &mut instrs, mgr);
	let value = into_reg(&instr.value, &mut instrs, mgr);
	instrs.push(IBinInstr::new(SW, value, (0, addr).into()));
	Ok(instrs)
}

pub fn riscv_load(
	instr: &llvm::llvminstr::LoadInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	if instr.addr.is_global() {
		let name = instr.addr.unwrap_temp().unwrap().name;
		let rd = mgr.get(&instr.target);
		instrs.push(IBinInstr::new(LA, rd, Label::new(name).into()));
	} else {
		let addr = into_reg(&instr.addr, &mut instrs, mgr);
		let rd = mgr.get(&instr.target);
		instrs.push(IBinInstr::new(LW, rd, (0, addr).into()));
	}
	Ok(instrs)
}

pub fn riscv_gep(
	instr: &llvm::llvminstr::GEPInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let rd = mgr.get(&instr.target);
	let (lhs, rhs) = (&instr.addr, &instr.offset);
	get_arith(rd, llvm::ArithOp::AddD, lhs, rhs, &mut instrs, mgr);
	Ok(instrs)
}

pub fn riscv_call(
	instr: &llvm::llvminstr::CallInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	// caller-saved
	let mut instrs: RiscvInstrSet = Vec::new();
	let mut end_instrs: RiscvInstrSet = Vec::new();

	instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-112).into()));
	CALLER_SAVE.iter().skip(1).enumerate().for_each(|(index, &reg)| {
		// TODO: 使用寄存器进行 caller-saved
		let instr =
			IBinInstr::new(SD, reg.into(), ((index * 8) as i32, SP.into()).into());
		instrs.push(instr);
		let instr =
			IBinInstr::new(LD, reg.into(), ((index * 8) as i32, SP.into()).into());
		end_instrs.push(instr);
	});

	// load parameters
	for (&reg, (_, val)) in PARAMETER_REGS.iter().zip(instr.params.iter()) {
		let rd = mgr.new_pre_color_temp(reg);
		get_arith(rd, llvm::ArithOp::Add, val, &0.into(), &mut instrs, mgr);
	}

	let cnt = max(0, instr.params.len() as i32 - 8) * 8; // 64 位的
	if cnt > 0 {
		instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-cnt).into()));
	}
	for (index, (_, val)) in instr.params.iter().skip(8).enumerate() {
		let value = into_reg(val, &mut instrs, mgr);
		instrs.push(IBinInstr::new(
			SD,
			value,
			((index * 8) as i32, SP.into()).into(),
		));
	}

	instrs.push(CallInstr::new(instr.func.clone()));

	if cnt > 0 {
		instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), cnt.into()));
	}
	instrs.extend(end_instrs);
	instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), 112.into()));

	// if !instr.target.var_type.is_void() {
	let ret_val = mgr.new_pre_color_temp(A0);
	let ret_instr = RTriInstr::new(Add, ret_val, A0.into(), X0.into());
	instrs.push(ret_instr);
	let rd = mgr.get(&instr.target);
	let instr = RTriInstr::new(Add, rd, ret_val, X0.into());
	instrs.push(instr);
	// }

	Ok(instrs)
}
