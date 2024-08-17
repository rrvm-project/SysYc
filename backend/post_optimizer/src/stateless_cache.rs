use std::{cell::RefCell, rc::Rc};

use instruction::riscv::reg::{
	Fa0, Fa1, Fa2, Fa3, Fa4, Fa5, Fa6, Fa7, A0, A1, A2, A3, A4, A5, A6, A7, T0,
	T1, T2, T3, T4, T5, T6,
};

use instruction::riscv::riscvop::IBinInstrOp::{
    SW, FSW, LW, FLW, SD, LD
};

use instruction::riscv::{
	prelude::{BranInstr, IBinInstr},
	value::RiscvTemp::{self, PhysReg},
};
use rrvm::{cfg::BasicBlock, program::RiscvProgram, RiscvNode};
use utils::label;

pub fn add_cache(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		if !func.need_cache {
			continue;
		}

		let entry = func.cfg.get_entry();

		dbg!(entry.borrow().id);
		for instr in &entry.borrow().instrs {
			eprintln!("{}", instr);
		}

		let mut hasher = func.new_basicblock(1f64);
		let haser_id = hasher.id;
		hasher.id = entry.borrow().id;
		entry.borrow_mut().id = haser_id;

		let old_label = entry.borrow().label();

		let test_instr = IBinInstr {
			op: SW.into(),
			rd: T0.into(),
			rs1: 3.into(),
		};

		hasher.push(Box::new(test_instr));
		hasher.jump_instr = Some(Box::new(BranInstr {
			op: instruction::riscv::riscvop::BranInstrOp::Beq,
			rs1: RiscvTemp::PhysReg(instruction::riscv::reg::RiscvReg::X0),
			rs2: RiscvTemp::PhysReg(instruction::riscv::reg::RiscvReg::X0),
			to: instruction::riscv::value::RiscvImm::Label(old_label),
		}));

		let hasher = Rc::new(RefCell::new(hasher));
		func.cfg.blocks.push(hasher);

		let length = func.cfg.blocks.len();
		func.cfg.blocks.swap(0, length - 1);
		func.cfg.blocks.swap(0, 1);

		let entry = func.cfg.get_entry();

		dbg!(entry.borrow().id);
		for instr in &entry.borrow().instrs {
			eprintln!("{}", instr);
		}

		// todo!()
	}
}
