use std::cmp::max;

use llvm::{ArithOp, CompOp, ConvertOp, Value};
use utils::{
	errors::{Result, SysycError::*},
	math::{align16, is_pow2},
	Label,
};

use crate::{
	riscv::{reg::*, riscvinstr::*, riscvop::*, value::*},
	temp::TempManager,
	RiscvInstrSet,
};

pub fn load_imm(
	num: impl Into<i64>,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> RiscvTemp {
	let num: i64 = num.into();
	if num == 0 {
		return RiscvTemp::PhysReg(X0); // 代价不同、最小化代价
	}
	let rd = mgr.new_temp();
	instrs.push(IBinInstr::new(Li, rd, num.into()));
	rd
}

fn end_num(val: &Value) -> Option<i32> {
	match val {
		Value::Int(v) => is_lower(*v).then_some(*v),
		_ => None,
	}
}

fn into_reg(
	val: &llvm::Value,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> RiscvTemp {
	match val {
		Value::Int(num) => load_imm(*num, instrs, mgr),
		Value::Float(num) => load_imm(num.to_bits(), instrs, mgr),
		Value::Temp(temp) => mgr.get(temp),
	}
}

fn solve_mul(
	rd: RiscvTemp,
	lhs: RiscvTemp,
	num: i32,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> bool {
	if num == 0 {
		instrs.push(RTriInstr::new(Add, rd, X0.into(), X0.into()));
		return true;
	}
	let offset = num.trailing_zeros() as i32;
	if is_pow2(num) {
		instrs.push(ITriInstr::new(Slliw, rd, lhs, offset.into()));
		return true;
	}
	let base = num >> offset;
	let temp = mgr.new_temp();
	if is_pow2(base - 1) {
		let offset_temp = (base - 1).trailing_zeros() as i32;
		instrs.push(ITriInstr::new(Slliw, temp, lhs, offset_temp.into()));
		instrs.push(RTriInstr::new(Addw, rd, temp, lhs));
		instrs.push(ITriInstr::new(Slliw, rd, rd, offset.into()));
		true
	} else if is_pow2(base + 1) {
		let offset_temp = (base + 1).trailing_zeros() as i32;
		instrs.push(ITriInstr::new(Slliw, temp, lhs, offset_temp.into()));
		instrs.push(RTriInstr::new(Subw, rd, temp, lhs));
		instrs.push(ITriInstr::new(Slliw, rd, rd, offset.into()));
		true
	} else {
		false
	}
}

fn solve_div(
	rd: RiscvTemp,
	lhs: RiscvTemp,
	num: i32,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) {
	if is_pow2(num) {
		let l = num.trailing_zeros() as i32;
		let temp = mgr.new_temp();
		instrs.push(ITriInstr::new(Srliw, temp, lhs, 31.into()));
		instrs.push(RTriInstr::new(Sub, rd, lhs, temp));
		instrs.push(ITriInstr::new(Srai, rd, rd, l.into()));
		instrs.push(RTriInstr::new(Addw, rd, rd, temp));
	} else {
		let l = ((num - 1).ilog2() + 1) as i32;
		let m = (2147483649i64 << l) / num as i64;
		let temp = load_imm(m, instrs, mgr);
		instrs.push(RTriInstr::new(Mul, rd, temp, lhs));
		instrs.push(ITriInstr::new(Srliw, temp, lhs, 31.into()));
		instrs.push(RTriInstr::new(Sub, rd, rd, temp));
		instrs.push(ITriInstr::new(Srai, rd, rd, (l + 31).into()));
		instrs.push(RTriInstr::new(Addw, rd, rd, temp));
	}
}

fn solve_rem(
	rd: RiscvTemp,
	lhs: RiscvTemp,
	num: i32,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) {
	if is_pow2(num) {
		let temp = mgr.new_temp();
		let l = num.trailing_zeros() as i32;
		if l == 1 {
			instrs.push(ITriInstr::new(Srliw, temp, lhs, 31.into()));
		} else {
			instrs.push(ITriInstr::new(Slli, temp, lhs, 1.into()));
			instrs.push(ITriInstr::new(Srli, temp, temp, (64 - l).into()));
		}
		instrs.push(RTriInstr::new(Add, temp, temp, lhs));
		if is_lower(-num) {
			instrs.push(ITriInstr::new(Andi, temp, temp, (-num).into()));
		} else {
			let imm = load_imm(-num, instrs, mgr);
			instrs.push(RTriInstr::new(And, temp, temp, imm));
		}
		instrs.push(RTriInstr::new(Sub, rd, lhs, temp));
	} else {
		solve_div(rd, lhs, num, instrs, mgr);
		let temp = load_imm(num, instrs, mgr);
		instrs.push(RTriInstr::new(Mul, rd, rd, temp));
		instrs.push(RTriInstr::new(Subw, rd, lhs, rd));
	}
}

fn get_arith(
	rd: RiscvTemp,
	op: ArithOp,
	lhs: &Value,
	rhs: &Value,
	instrs: &mut RiscvInstrSet,
	mgr: &mut TempManager,
) -> Result<()> {
	let lhs = into_reg(lhs, instrs, mgr);
	match (op, end_num(rhs), rhs) {
		(ArithOp::Sub, _, _) if lhs.is_zero() => {
			let rhs = into_reg(rhs, instrs, mgr);
			instrs.push(RBinInstr::new(Negw, rd, rhs));
		}
		(ArithOp::Sub, Some(num), _) if is_lower(-num) => {
			instrs.push(ITriInstr::new(Addiw, rd, lhs, (-num).into()));
		}
		(ArithOp::Mul, _, Value::Int(num))
			if solve_mul(rd, lhs, num.abs(), instrs, mgr) =>
		{
			if *num < 0 {
				instrs.push(RBinInstr::new(Negw, rd, rd));
			}
		}
		(ArithOp::Div, _, Value::Int(num)) => match num {
			0 => return Err(SemanticError("divided by zero".to_string())),
			1 => instrs.push(RTriInstr::new(Add, rd, lhs, X0.into())),
			-1 => instrs.push(RBinInstr::new(Neg, rd, lhs)),
			_ => {
				solve_div(rd, lhs, num.abs(), instrs, mgr);
				if *num < 0 {
					instrs.push(RBinInstr::new(Negw, rd, rd));
				}
			}
		},
		(ArithOp::Rem, _, Value::Int(num)) => match num {
			0 => return Err(SemanticError("moduled by zero".to_string())),
			1 | -1 => instrs.push(RTriInstr::new(Add, rd, X0.into(), X0.into())),
			_ => solve_rem(rd, lhs, num.abs(), instrs, mgr),
		},
		(_, Some(num), _) if can_to_iop(&op) => {
			instrs.push(ITriInstr::new(to_iop(&op), rd, lhs, num.into()));
		}
		_ => {
			let rhs = into_reg(rhs, instrs, mgr);
			instrs.push(RTriInstr::new(to_rop(&op), rd, lhs, rhs));
		}
	};
	Ok(())
}

pub fn riscv_arith(
	instr: &llvm::ArithInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let (lhs, rhs) = (&instr.lhs, &instr.rhs);
	let target = mgr.get(&instr.target);
	get_arith(target, instr.op, lhs, rhs, &mut instrs, mgr)?;
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
	instr: &llvm::CompInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let (lhs, rhs) = (&instr.lhs, &instr.rhs);
	let target = mgr.get(&instr.target);
	match &instr.op {
		CompOp::EQ | CompOp::OEQ => {
			let tmp = mgr.new_temp();
			get_arith(tmp, ArithOp::Xor, lhs, rhs, &mut instrs, mgr)?;
			instrs.push(ITriInstr::new(Sltiu, target, tmp, 1.into()));
		}
		CompOp::NE | CompOp::ONE => {
			let tmp = mgr.new_temp();
			get_arith(tmp, ArithOp::Xor, lhs, rhs, &mut instrs, mgr)?;
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
	instr: &llvm::ConvertInstr,
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
	instr: &llvm::JumpInstr,
	_mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let to = instr.target.clone().into();
	instrs.push(BranInstr::new(Beq, X0.into(), X0.into(), to));
	Ok(instrs)
}

pub fn riscv_cond(
	instr: &llvm::JumpCondInstr,
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
	_instr: &llvm::PhiInstr,
	_mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	unreachable!("phi instruction should be solved before instruction selcetion")
}

pub fn riscv_ret(
	instr: &llvm::RetInstr,
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
	instr: &llvm::AllocInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let size = &instr.length;
	if let Some(num) = end_num(size) {
		if num % 16 != 0 {
			return Err(RiscvGenError("stack should be aligned to 16".to_string()));
		}
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
	instr: &llvm::StoreInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let addr = into_reg(&instr.addr, &mut instrs, mgr);
	let value = into_reg(&instr.value, &mut instrs, mgr);
	instrs.push(IBinInstr::new(SW, value, (0, addr).into()));
	Ok(instrs)
}

pub fn riscv_load(
	instr: &llvm::LoadInstr,
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
	instr: &llvm::GEPInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	let rd = mgr.get(&instr.target);
	let (lhs, rhs) = (&instr.addr, &instr.offset);
	get_arith(rd, llvm::ArithOp::AddD, lhs, rhs, &mut instrs, mgr)?;
	Ok(instrs)
}

pub fn riscv_call(
	instr: &llvm::CallInstr,
	mgr: &mut TempManager,
) -> Result<RiscvInstrSet> {
	let mut instrs: RiscvInstrSet = Vec::new();
	instrs.push(TemporayInstr::new(Save));
	let size = align16(max(0, instr.params.len() as i32 - 8) * 8);
	if size > 0 {
		instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), (-size).into()));
	}
	for (index, (_, val)) in instr.params.iter().skip(8).enumerate() {
		let value = into_reg(val, &mut instrs, mgr);
		instrs.push(IBinInstr::new(
			SD,
			value,
			((index * 8) as i32, SP.into()).into(),
		));
	}
	// load parameters
	let mut params = Vec::new();
	for (&reg, (_, val)) in PARAMETER_REGS.iter().zip(instr.params.iter()) {
		let rd = mgr.new_pre_color_temp(reg);
		params.push(reg.into());
		get_arith(rd, llvm::ArithOp::AddD, val, &0.into(), &mut instrs, mgr)?;
	}

	instrs.push(CallInstr::new(instr.func.clone(), params));

	if size > 0 {
		instrs.push(ITriInstr::new(Addi, SP.into(), SP.into(), size.into()));
	}
	instrs.push(TemporayInstr::new(Restore));
	let ret_val = mgr.new_pre_color_temp(A0);
	instrs.push(RTriInstr::new(Add, ret_val, A0.into(), X0.into()));
	let rd = mgr.get(&instr.target);
	if !instr.var_type.is_void() {
		instrs.push(RTriInstr::new(Add, rd, ret_val, X0.into()));
	}
	Ok(instrs)
}
