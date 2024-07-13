#![allow(clippy::new_ret_no_self)]
use super::{
	reg::RiscvReg::{self, *},
	riscvinstr::*,
	riscvop::*,
	utils::*,
	value::*,
	virt_mem::VirtAddr,
};
use crate::temp::Temp;
use std::{
	collections::HashMap,
	fmt::{Display, Formatter, Result},
};
use utils::{mapper::LabelMapper, Label};

impl Display for RTriInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		if self.is_move() {
			let val = if self.rs1.is_zero() {
				self.rs2
			} else {
				self.rs1
			};
			write!(f, "  mv {}, {}", self.rd, val)
		} else {
			write!(f, "  {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2)
		}
	}
}

impl RiscvInstrTrait for RTriInstr {
	fn map_src_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rs1, map);
		map_temp(&mut self.rs2, map);
	}
	fn map_dst_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rd, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		vec![self.rs1, self.rs2]
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![self.rd]
	}
	fn is_move(&self) -> bool {
		matches!(self.op, Add | Addw | Or | Xor)
			&& (self.rs1.is_zero() || self.rs2.is_zero())
	}
	fn useless(&self) -> bool {
		match (&self.op, &self.rd, &self.rs1, &self.rs2) {
			(Add | Addw | Xor | Or, PhysReg(x), PhysReg(y), PhysReg(z)) => {
				x == y && *z == X0 || x == z && *y == X0
			}
			(Srl | Sra | Slt | Sltu, PhysReg(x), PhysReg(y), PhysReg(X0)) => x == y,
			_ => false,
		}
	}
}

impl RTriInstr {
	pub fn new(
		op: RTriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
	) -> RiscvInstr {
		Box::new(Self { op, rs1, rs2, rd })
	}
}

impl Display for ITriInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match self.op {
			Addi | Addiw | Xori | Ori if self.rs1.is_zero() => {
				write!(f, "  li {}, {}", self.rd, self.rs2)
			}
			Addi | Addiw | Xori | Ori if self.rs2.is_zero() => {
				write!(f, "  mv {}, {}", self.rd, self.rs1)
			}
			_ => write!(f, "  {} {}, {}, {}", self.op, self.rd, self.rs1, self.rs2),
		}
	}
}

impl RiscvInstrTrait for ITriInstr {
	fn map_src_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rs1, map);
		map_imm_temp(&mut self.rs2, map);
	}
	fn map_dst_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rd, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		[vec![self.rs1], unwarp_imms(vec![&self.rs2])].concat()
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![self.rd]
	}
	fn is_move(&self) -> bool {
		matches!(self.op, Addi | Xori | Ori if self.rs2.is_zero())
	}
	fn map_label(&mut self, map: &mut LabelMapper) {
		map_imm_label(&mut self.rs2, map);
	}
	fn useless(&self) -> bool {
		match (&self.op, &self.rd, &self.rs1, &self.rs2) {
			(Addi, PhysReg(x), PhysReg(y), Int(0)) if x == y => true,
			(Xori, PhysReg(x), PhysReg(y), Int(0)) if x == y => true,
			(Ori, PhysReg(x), PhysReg(y), Int(0)) if x == y => true,
			_ => false,
		}
	}
	fn get_increment(&self) -> IncrementType {
		match self.op {
			Addi | Addiw => match self.rs2 {
				RiscvImm::Int(v) => IncrementType::Int(v),
				RiscvImm::LongLong(v) => IncrementType::LongLong(v),
				_ => IncrementType::None,
			},
			_ => IncrementType::None,
		}
	}
}

impl ITriInstr {
	pub fn new(
		op: ITriInstrOp,
		rd: RiscvTemp,
		rs1: RiscvTemp,
		rs2: RiscvImm,
	) -> RiscvInstr {
		Box::new(Self { op, rs1, rs2, rd })
	}
}

impl Display for IBinInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "  {} {}, {}", self.op, self.rd, self.rs1)
	}
}

impl RiscvInstrTrait for IBinInstr {
	fn map_src_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_imm_temp(&mut self.rs1, map);
		if matches!(self.op, SB | SH | SW | SD | FSW | FSD) {
			map_temp(&mut self.rd, map);
		}
	}
	fn map_dst_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		if matches!(self.op, SB | Li | Lui | LD | LW | LWU | LA | FLW | FLD) {
			map_temp(&mut self.rd, map);
		}
	}
	fn map_virt_mem(&mut self, map: &HashMap<VirtAddr, (i32, RiscvTemp)>) {
		map_virt_mem(&mut self.rs1, map);
	}
	fn get_virt_mem_write(&self) -> Option<VirtAddr> {
		match self.op {
			SB | SH | SW | SD => self.rs1.to_virt_mem(),
			_ => None,
		}
	}
	fn get_virt_mem_read(&self) -> Option<VirtAddr> {
		match self.op {
			Li | Lui | LD | LW | LWU | LA => self.rs1.to_virt_mem(),
			_ => None,
		}
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		match self.op {
			Li | Lui | LD | LW | LWU | LA | FLW | FLD => vec![self.rd],
			SB | SH | SW | SD | FSW | FSD => vec![],
		}
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		[
			match self.op {
				Li | Lui | LD | LW | LWU | LA | FLW | FLD => vec![],
				SB | SH | SW | SD | FSW | FSD => vec![self.rd],
			},
			unwarp_imms(vec![&self.rs1]),
		]
		.concat()
	}
	fn map_label(&mut self, map: &mut LabelMapper) {
		if self.op != LA {
			map_imm_label(&mut self.rs1, map);
		}
	}
	fn get_imm(&self) -> Option<RiscvImm> {
		Some(self.rs1.clone())
	}
	fn is_load(&self) -> Option<bool> {
		match self.op {
			Li | Lui | LD | LW | LWU | LA => Some(true),
			SB | SH | SW | SD => Some(false),
		}
	}
	fn is_store(&self) -> Option<bool> {
		match self.op {
			Li | Lui | LD | LW | LWU | LA => Some(false),
			SB | SH | SW | SD => Some(true),
		}
	}
}

impl IBinInstr {
	pub fn new(op: IBinInstrOp, rd: RiscvTemp, rs1: RiscvImm) -> RiscvInstr {
		Box::new(Self { op, rs1, rd })
	}
}

impl Display for LabelInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}:", self.label)
	}
}

impl RiscvInstrTrait for LabelInstr {
	fn map_label(&mut self, map: &mut LabelMapper) {
		map_label(&mut self.label, map);
	}
	fn get_write_label(&self) -> Option<Label> {
		Some(self.label.clone())
	}
}

impl LabelInstr {
	pub fn new(label: Label) -> RiscvInstr {
		Box::new(Self { label })
	}
}

impl Display for RBinInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		if self.op == Float2Int {
			write!(f, "  {} {}, {}, rtz", self.op, self.rd, self.rs1)
		} else {
			write!(f, "  {} {}, {}", self.op, self.rd, self.rs1)
		}
	}
}

impl RiscvInstrTrait for RBinInstr {
	fn map_src_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rs1, map);
	}
	fn map_dst_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rd, map);
	}
	fn is_move(&self) -> bool {
		matches!(self.op, Mv | FMv)
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		vec![self.rs1]
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![self.rd]
	}
	fn useless(&self) -> bool {
		self.is_move() && self.rd == self.rs1
	}
}

impl RBinInstr {
	pub fn new(op: RBinInstrOp, rd: RiscvTemp, rs1: RiscvTemp) -> RiscvInstr {
		Box::new(Self { op, rs1, rd })
	}
}

impl Display for BranInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		if self.op == Beq && self.rs1.is_zero() && self.rs2.is_zero() {
			write!(f, "  j {}", self.to)
		} else {
			write!(f, "  {} {}, {}, {}", self.op, self.rs1, self.rs2, self.to)
		}
	}
}

impl RiscvInstrTrait for BranInstr {
	fn map_src_temp(&mut self, map: &HashMap<Temp, RiscvTemp>) {
		map_temp(&mut self.rs1, map);
		map_temp(&mut self.rs2, map);
		map_imm_temp(&mut self.to, map);
	}
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		[vec![self.rs1, self.rs2], unwarp_imms(vec![&self.to])].concat()
	}
	fn map_label(&mut self, map: &mut LabelMapper) {
		map_imm_label(&mut self.to, map);
	}
	fn get_read_label(&self) -> Option<Label> {
		match &self.to {
			RiscvImm::Label(label) => Some(label.clone()),
			_ => unreachable!(),
		}
	}
}

impl BranInstr {
	pub fn new(
		op: BranInstrOp,
		rs1: RiscvTemp,
		rs2: RiscvTemp,
		to: RiscvImm,
	) -> RiscvInstr {
		Box::new(Self { op, rs1, rs2, to })
	}

	pub fn new_j(to: RiscvImm) -> RiscvInstr {
		Box::new(Self {
			op: Beq,
			rs1: X0.into(),
			rs2: X0.into(),
			to,
		})
	}
}

impl Display for NoArgInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "  {}", self.op)
	}
}

impl RiscvInstrTrait for NoArgInstr {
	fn is_ret(&self) -> bool {
		true
	}
}

impl NoArgInstr {
	pub fn new(op: NoArgInstrOp) -> RiscvInstr {
		Box::new(Self { op })
	}
}

impl Display for CallInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "  call {}", self.func_label)
	}
}

impl RiscvInstrTrait for CallInstr {
	fn get_riscv_read(&self) -> Vec<RiscvTemp> {
		self.params.clone()
	}
	fn get_riscv_write(&self) -> Vec<RiscvTemp> {
		vec![RA.into()]
	}
	fn is_call(&self) -> bool {
		true
	}
}

impl CallInstr {
	pub fn new(func_label: Label, params: Vec<RiscvTemp>) -> RiscvInstr {
		Box::new(Self { func_label, params })
	}
}

impl Display for TemporayInstr {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(
			f,
			"  {}[{}]",
			self.op,
			self.lives.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
		)
	}
}

impl RiscvInstrTrait for TemporayInstr {
	fn get_temp_op(&self) -> Option<TemporayInstrOp> {
		Some(self.op)
	}
	fn set_lives(&mut self, lives: Vec<RiscvReg>) {
		self.lives = lives
	}
	fn get_lives(&self) -> Vec<RiscvReg> {
		self.lives.clone()
	}
	fn get_temp_type(&self) -> llvm::VarType {
		self.var_type
	}
}

impl TemporayInstr {
	pub fn new(op: TemporayInstrOp, var_type: llvm::VarType) -> RiscvInstr {
		Box::new(Self {
			op,
			var_type,
			lives: Vec::new(),
		})
	}
}
