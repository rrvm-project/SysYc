use std::{collections::HashMap, fmt::Display};
use sysyc_derive::UseTemp;
use utils::{mapper::LabelMapper, InstrTrait, Label, UseTemp, RTN};

use crate::temp::Temp;

use super::{reg::RiscvReg, riscvop::*, value::*, virt_mem::VirtAddr};

pub type RiscvInstr = Box<dyn RiscvInstrTrait>;

pub enum RiscvInstrVariant<'a> {
	RTriInstr(&'a RTriInstr),
	ITriInstr(&'a ITriInstr),
	IBinInstr(&'a IBinInstr),
	RBinInstr(&'a RBinInstr),
	LabelInstr(&'a LabelInstr),
	BranInstr(&'a BranInstr),
	NoArgInstr(&'a NoArgInstr),
	CallInstr(&'a CallInstr),
	TemporayInstr(&'a TemporayInstr),
	PCRelLabelInstr(&'a PCRelLabelInstr),
}
pub trait CloneRiscvInstr {
	fn clone_box(&self) -> Box<dyn RiscvInstrTrait>;
}

impl<T> CloneRiscvInstr for T
where
	T: 'static + RiscvInstrTrait + Clone,
{
	fn clone_box(&self) -> Box<dyn RiscvInstrTrait> {
		Box::new(self.clone())
	}
}

impl Clone for RiscvInstr {
	fn clone(&self) -> Self {
		self.clone_box()
	}
}
pub trait RiscvInstrTrait:
	Display + UseTemp<Temp> + CloneRiscvInstr + RTN
{
	fn map_src_temp(&mut self, _map: &HashMap<Temp, RiscvTemp>) {}
	fn map_dst_temp(&mut self, _map: &HashMap<Temp, RiscvTemp>) {}
	fn map_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		self.map_src_temp(map);
		self.map_dst_temp(map);
	}
	fn map_virt_mem(&mut self, _map: &HashMap<VirtAddr, (i32, RiscvTemp)>) {}
	fn set_lives(&mut self, _lives: Vec<RiscvReg>) {}
	fn get_lives(&self) -> Vec<RiscvReg> {
		Vec::new()
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		Vec::new()
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		Vec::new()
	}
	fn get_virt_mem_write(&self) -> Option<VirtAddr> {
		None
	}
	fn get_virt_mem_read(&self) -> Option<VirtAddr> {
		None
	}
	fn get_read_label(&self) -> Option<Label> {
		None
	}
	fn get_write_label(&self) -> Option<Label> {
		None
	}
	fn get_imm(&self) -> Option<RiscvImm> {
		None
	}
	fn is_move(&self) -> bool {
		false
	}
	fn is_ret(&self) -> bool {
		false
	}
	fn is_call(&self) -> bool {
		false
	}
	fn is_load(&self) -> bool {
		false
	}
	fn is_store(&self) -> bool {
		false
	}
	fn map_label(&mut self, _map: &mut LabelMapper) {}
	fn useless(&self) -> bool {
		false
	}
	fn get_temp_op(&self) -> Option<TemporayInstrOp> {
		None
	}
	fn get_temp_type(&self) -> llvm::VarType {
		unreachable!()
	}
	fn is_save(&self) -> bool {
		false
	}
	fn is_restore(&self) -> bool {
		false
	}
	fn is_branch(&self) -> bool {
		false
	}
	fn get_cmp_op(&self) -> Option<BranInstrOp> {
		None
	}
	fn get_variant(&self) -> RiscvInstrVariant;
	fn is_fdiv(&self) -> bool {
		false
	}
}

impl UseTemp<Temp> for RiscvInstr {
	fn get_read(&self) -> Vec<Temp> {
		self.as_ref().get_read()
	}
	fn get_write(&self) -> Option<Temp> {
		self.as_ref().get_write()
	}
}

impl InstrTrait<Temp> for RiscvInstr {
	fn is_call(&self) -> bool {
		self.as_ref().is_call()
	}
	fn is_branch(&self) -> bool {
		self.as_ref().is_branch()
	}
}
impl RTN for RiscvInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		self.as_ref().get_rtn_array()
	}
}
#[derive(UseTemp, Clone)]
pub struct RTriInstr {
	pub op: RTriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
}
impl RTN for RTriInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		match self.op {
			RTriInstrOp::Mul => [0, 0, 1, 0, 3],
			RTriInstrOp::Mulw => [0, 0, 1, 0, 3],
			RTriInstrOp::Div => [0, 0, 1, 0, 12],
			RTriInstrOp::Divw => [0, 0, 1, 0, 12],
			RTriInstrOp::Rem => [0, 0, 1, 0, 12],
			RTriInstrOp::Remw => [0, 0, 1, 0, 12],
			RTriInstrOp::Fadd => [0, 0, 0, 1, 5],
			RTriInstrOp::Fsub => [0, 0, 0, 1, 5],
			RTriInstrOp::Fmul => [0, 0, 0, 1, 5],
			RTriInstrOp::Fdiv => [0, 0, 0, 1, 27],
			RTriInstrOp::Feq => [0, 0, 0, 1, 4],
			RTriInstrOp::Flt => [0, 0, 0, 1, 4],
			RTriInstrOp::Fle => [0, 0, 0, 1, 4],
			_ => [0, 0, 0, 0, 1],
		}
	}
}
#[derive(UseTemp, Clone)]
pub struct ITriInstr {
	pub op: ITriInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvImm,
}
impl RTN for ITriInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 0, 0, 0, 1]
	}
}
#[derive(UseTemp, Clone)]
pub struct IBinInstr {
	pub op: IBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvImm,
}
impl RTN for IBinInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		match self.op {
			IBinInstrOp::LD => [1, 0, 0, 0, 3],
			IBinInstrOp::LA => [1, 0, 0, 0, 3],
			IBinInstrOp::Li => [0, 0, 0, 0, 1],
			IBinInstrOp::LW => [1, 0, 0, 0, 3],
			IBinInstrOp::LWU => [1, 0, 0, 0, 3],
			IBinInstrOp::FSD => [0, 0, 0, 1, 4],
			IBinInstrOp::FSW => [0, 0, 0, 1, 4],
			IBinInstrOp::FLD => [0, 0, 0, 1, 2],
			IBinInstrOp::FLW => [0, 0, 0, 1, 2],
			_ => [1, 0, 0, 0, 1],
		}
	}
}
#[derive(UseTemp, Clone)]
pub struct RBinInstr {
	pub op: RBinInstrOp,
	pub rd: RiscvTemp,
	pub rs1: RiscvTemp,
}
impl RTN for RBinInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		match self.op {
			RBinInstrOp::Float2Int => [0, 0, 0, 1, 4],
			RBinInstrOp::Int2Float => [0, 0, 0, 1, 2],
			RBinInstrOp::FMv => [0, 0, 0, 1, 2],
			RBinInstrOp::MvInt2Float => [0, 0, 0, 1, 2],
			_ => [0, 0, 0, 0, 1],
		}
	}
}
#[derive(UseTemp, Clone)]
pub struct LabelInstr {
	pub label: Label,
}
impl RTN for LabelInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 0, 0, 0, 0]
	}
}
#[derive(UseTemp, Clone)]
pub struct BranInstr {
	pub op: BranInstrOp,
	pub rs1: RiscvTemp,
	pub rs2: RiscvTemp,
	pub to: RiscvImm,
}
impl RTN for BranInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 1, 0, 0, 1]
	}
}
#[derive(UseTemp, Clone)]
pub struct NoArgInstr {
	pub op: NoArgInstrOp,
}
impl RTN for NoArgInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 1, 0, 0, 1]
	}
}
#[derive(UseTemp, Clone)]
pub struct CallInstr {
	pub func_label: Label,
	pub params: Vec<RiscvTemp>,
}
impl RTN for CallInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 1, 0, 0, 1]
	}
}
#[derive(UseTemp, Clone)]
pub struct TemporayInstr {
	pub op: TemporayInstrOp,
	pub var_type: llvm::VarType,
	pub lives: Vec<RiscvReg>,
}

impl RTN for TemporayInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 0, 0, 0, 1]
	}
}

#[derive(UseTemp, Clone)]
pub struct PCRelLabelInstr {
	pub label: String,
}
impl RTN for PCRelLabelInstr {
	fn get_rtn_array(&self) -> [i32; 5] {
		[0, 0, 0, 0, 0]
	}
}
