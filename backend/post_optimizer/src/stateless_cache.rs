use std::{cell::RefCell, rc::Rc};

use instruction::riscv::{
	prelude::{ITriInstr, NoArgInstr, RBinInstr, RTriInstr},
	reg::{
		Fa0, Fa1, Fa2, Fa3, Fa4, Fa5, Fa6, Fa7,
		RiscvReg::{self, SP, X0},
		A0, A1, A2, A3, A4, A5, A6, A7, T0, T1, T2, T3, T4, T5, T6,
	},
	riscvop::{IBinInstrOp::FSW, ITriInstrOp::Addi},
	value::{
		RiscvImm::{LongLong, OffsetReg},
		RiscvNumber,
	},
};

use instruction::riscv::riscvop::{
	BranInstrOp::Beq,
	IBinInstrOp::{Li, FLW, LA, LD, LW, SD, SW},
	ITriInstrOp::{Addiw, Andi, Slli, Slliw, Srli},
	RBinInstrOp::{Mv, MvFloat2Int},
	RTriInstrOp::{Add, Or, Xor},
};

use instruction::riscv::prelude::{BranInstr, IBinInstr};

use instruction::riscv::riscvinstr::RiscvInstr;

use llvm::VarType;
use rrvm::program::{RiscvFunc, RiscvProgram};

pub fn get_using_reg(func: &RiscvFunc) -> Option<(RiscvReg, Vec<RiscvReg>)> {
	dbg!(&func.name);
	let mut int_regs = vec![A0, A1, A2, A3, A4, A5, A6, A7].into_iter();
	let mut float_regs = vec![Fa0, Fa1, Fa2, Fa3, Fa4, Fa5, Fa6, Fa7].into_iter();

	let mut result = vec![];
	for para in func.params.iter() {
		match para.get_type() {
			VarType::I32 => {
				result.push(int_regs.next()?);
			}
			VarType::F32 => {
				result.push(float_regs.next()?);
			}
			_ => {
				break;
			}
		}
	}

	let return_reg = match func.ret_type {
		VarType::I32 => A0,
		VarType::F32 => Fa0,
		_ => {
			return None;
		}
	};

	Some((return_reg, result))
}

pub fn add_cache(program: &mut RiscvProgram) {
	fn get_mod(reg: RiscvReg) -> RiscvInstr {
		match utils::CACHE_SIZE {
			4 | 8 => Box::new(ITriInstr {
				op: Andi,
				rd: reg.into(),
				rs1: reg.into(),
				rs2: ((utils::CACHE_SIZE - 1) as i32).into(),
			}),
			_ => unreachable!(),
		}
	}

	for func in program.funcs.iter_mut() {
		if !func.need_cache {
			continue;
		}
		let get_arg_hash_name =
			format!("{}_{}_ARG", utils::CACHE_PREFIX, func.name.as_str());

		let get_return_name =
			format!("{}_{}_RETURN", utils::CACHE_PREFIX, func.name.as_str());

		let get_begin_name =
			format!("{}_{}_BEGIN", utils::CACHE_PREFIX, func.name.as_str());

		let entry = func.cfg.get_entry();

		let mut hasher = func.new_basicblock(1f64);
		let haser_id = hasher.id;
		hasher.id = entry.borrow().id;
		entry.borrow_mut().id = haser_id;
		let old_entry_label = entry.borrow().label();

		let func_type = func.ret_type;
		let store_result = |tmp: RiscvReg| {
			if func_type.is_float() {
				Box::new(IBinInstr {
					op: FSW,
					rd: Fa0.into(),
					rs1: OffsetReg(RiscvNumber::Int(0), tmp.into()),
				})
			} else {
				Box::new(IBinInstr {
					op: SW,
					rd: A0.into(),
					rs1: OffsetReg(RiscvNumber::Int(0), tmp.into()),
				})
			}
		};

		let (return_reg, mut args) = get_using_reg(func).unwrap();

		let mut middles = vec![];

		let mut reamin_weight = 1f64;
		let mut weight_for_success = 0f64;

		let mut success_return = func.new_basicblock(1f64);
		for _ in 0..utils::CACHE_SIZE {
			let mut new_middle = func.new_basicblock(reamin_weight * 0.95);
			weight_for_success += reamin_weight * 0.05;
			reamin_weight *= 0.95;

			new_middle.push(Box::new(RBinInstr {
				op: Mv,
				rd: T1.into(),
				rs1: T3.into(),
			}));

			new_middle.push(Box::new(ITriInstr {
				op: Slliw,
				rd: T2.into(),
				rs1: T1.into(),
				rs2: 3.into(),
			}));

			new_middle.push(Box::new(RTriInstr {
				op: Add,
				rd: T2.into(),
				rs1: T2.into(),
				rs2: T5.into(),
			}));

			new_middle.push(Box::new(IBinInstr {
				op: LD,
				rd: T4.into(),
				rs1: OffsetReg(RiscvNumber::Int(0), T2.into()),
			}));

			new_middle.push(Box::new(ITriInstr {
				op: Addiw,
				rd: T3.into(),
				rs1: T1.into(),
				rs2: 1.into(),
			}));

			new_middle.push(get_mod(T3));

			new_middle.instrs.push(Box::new(BranInstr {
				op: Beq,
				rs1: T4.into(),
				rs2: T0.into(),
				to: success_return.label().into(),
			}));

			middles.push(new_middle);
		}

		success_return.weight = weight_for_success;

		success_return.push(Box::new(IBinInstr {
			op: LA,
			rd: T5.into(),
			rs1: utils::Label::new(get_return_name.clone()).into(),
		}));

		success_return.push(Box::new(ITriInstr {
			op: Slli,
			rd: T1.into(),
			rs1: T1.into(),
			rs2: 2.into(), // Return array are 4Bytes each
		}));

		success_return.push(Box::new(RTriInstr {
			op: Add,
			rd: T5.into(),
			rs1: T1.into(),
			rs2: T5.into(), // Return array are 4Bytes each
		}));

		success_return.push(Box::new(IBinInstr {
			op: if func.ret_type.is_float() { FLW } else { LW },
			rd: return_reg.into(),
			rs1: OffsetReg(RiscvNumber::Int(0), T5.into()),
		}));

		let mut go_to_normal = func.new_basicblock(reamin_weight);

		go_to_normal.push(Box::new(ITriInstr {
			op: Addi,
			rd: SP.into(),
			rs1: SP.into(),
			rs2: (-16).into(),
		}));

		go_to_normal.push(Box::new(IBinInstr {
			op: SD,
			rd: T0.into(),
			rs1: OffsetReg(RiscvNumber::Int(0), SP.into()),
		}));

		hasher.push(Box::new(IBinInstr {
			op: Li,
			rd: T0.into(),
			rs1: LongLong(0x1926081747220717),
		}));

		let mut offsets = vec![11, 32, 27, 23, 19, 41, 37].into_iter().cycle();

		let mut another_args = args.clone().into_iter().cycle();
		another_args.next();
		let len = args.len();
		if args.len() != 1 {
			args.pop();
		}

		for item in args {
			match item.get_type() {
				instruction::temp::VarType::Int => hasher.push(Box::new(RTriInstr {
					op: Xor,
					rd: T0.into(),
					rs1: item.into(),
					rs2: T0.into(),
				})),
				instruction::temp::VarType::Float => {
					hasher.push(Box::new(RBinInstr {
						op: MvFloat2Int,
						rd: T1.into(),
						rs1: item.into(),
					}));
					hasher.push(Box::new(RTriInstr {
						op: Xor,
						rd: T0.into(),
						rs1: T1.into(),
						rs2: T0.into(),
					}));
				}
			}

			if len > 1 {
				let another = another_args.next().unwrap();

				match another.get_type() {
					instruction::temp::VarType::Int => hasher.push(Box::new(RBinInstr {
						op: Mv,
						rd: T2.into(),
						rs1: another.into(),
					})),
					instruction::temp::VarType::Float => {
						hasher.push(Box::new(RBinInstr {
							op: MvFloat2Int,
							rd: T2.into(),
							rs1: another.into(),
						}));
					}
				}

				let offset = offsets.next().unwrap();
				if offset != 0 {
					let remain_offset = 64 - offset;
					hasher.push(Box::new(RTriInstr {
						op: Add,
						rd: T0.into(),
						rs1: T0.into(),
						rs2: T2.into(),
					}));
					hasher.push(Box::new(ITriInstr {
						op: Slli,
						rd: T1.into(),
						rs1: T0.into(),
						rs2: offset.into(),
					}));
					hasher.push(Box::new(ITriInstr {
						op: Srli,
						rd: T0.into(),
						rs1: T0.into(),
						rs2: remain_offset.into(),
					}));

					hasher.push(Box::new(RTriInstr {
						op: Or,
						rd: T0.into(),
						rs1: T0.into(),
						rs2: T1.into(),
					}))
				}
			}
		}

		hasher.push(Box::new(IBinInstr {
			op: LA,
			rd: T6.into(),
			rs1: utils::Label::new(get_begin_name.clone()).into(),
		}));

		hasher.push(Box::new(IBinInstr {
			op: LA,
			rd: T5.into(),
			rs1: utils::Label::new(get_arg_hash_name.clone()).into(),
		}));

		hasher.push(Box::new(IBinInstr {
			op: LW,
			rd: T3.into(),
			rs1: OffsetReg(RiscvNumber::Int(0), T6.into()),
		}));

		//JUMP instrs
		success_return.jump_instr = Some(Box::new(NoArgInstr {
			op: instruction::riscv::riscvop::NoArgInstrOp::Ret,
		}));

		go_to_normal.jump_instr = Some(Box::new(BranInstr {
			op: Beq,
			rs1: X0.into(),
			rs2: X0.into(),
			to: old_entry_label.into(),
		}));

		hasher.jump_instr = Some(Box::new(BranInstr {
			op: Beq,
			rs1: X0.into(),
			rs2: X0.into(),
			to: middles[0].label().into(),
		}));

		for i in 1..utils::CACHE_SIZE {
			middles[i - 1].jump_instr = Some(Box::new(BranInstr {
				op: Beq,
				rs1: X0.into(),
				rs2: X0.into(),
				to: middles[i].label().into(),
			}));
		}

		middles[utils::CACHE_SIZE - 1].jump_instr = Some(Box::new(BranInstr {
			op: Beq,
			rs1: X0.into(),
			rs2: X0.into(),
			to: go_to_normal.label().into(),
		}));

		let before_return: Vec<RiscvInstr> = vec![
			Box::new(IBinInstr {
				op: LA,
				rd: T6.into(),
				rs1: utils::Label::new(get_begin_name.clone()).into(),
			}),
			Box::new(IBinInstr {
				op: LW,
				rd: T1.into(),
				rs1: OffsetReg(RiscvNumber::Int(0), T6.into()),
			}),
			Box::new(ITriInstr {
				op: Addiw,
				rd: T1.into(),
				rs1: T1.into(),
				rs2: (-1).into(),
			}),
			get_mod(T1),
			Box::new(IBinInstr {
				// T1: head(updated)
				op: SW,
				rd: T1.into(),
				rs1: OffsetReg(RiscvNumber::Int(0), T6.into()),
			}),
			Box::new(IBinInstr {
				// T0: hash
				op: LD,
				rd: T0.into(),
				rs1: OffsetReg(RiscvNumber::Int(0), SP.into()),
			}),
			Box::new(ITriInstr {
				op: Addi,
				rd: SP.into(),
				rs1: SP.into(),
				rs2: 16.into(),
			}),
			Box::new(IBinInstr {
				op: LA,
				rd: T5.into(),
				rs1: utils::Label::new(get_return_name.clone()).into(),
			}),
			Box::new(ITriInstr {
				op: Slli,
				rd: T4.into(),
				rs1: T1.into(),
				rs2: 2.into(), // Return array are 4Bytes each
			}),
			Box::new(RTriInstr {
				op: Add,
				rd: T4.into(),
				rs1: T5.into(),
				rs2: T4.into(),
			}),
			store_result(T4),
			Box::new(IBinInstr {
				op: LA,
				rd: T5.into(),
				rs1: utils::Label::new(get_arg_hash_name.clone()).into(),
			}),
			Box::new(ITriInstr {
				op: Slli,
				rd: T4.into(),
				rs1: T1.into(),
				rs2: 3.into(), // Hash array are 8Bytes each
			}),
			Box::new(RTriInstr {
				op: Add,
				rd: T4.into(),
				rs1: T5.into(),
				rs2: T4.into(),
			}),
			Box::new(IBinInstr {
				op: SD,
				rd: T0.into(),
				rs1: OffsetReg(RiscvNumber::Int(0), T4.into()),
			}),
		];

		for block in &mut func.cfg.blocks {
			let mut flag = false;
			if let Some(jump) = &block.borrow().jump_instr {
				if matches!(
					jump.get_variant(),
					instruction::riscv::prelude::RiscvInstrVariant::NoArgInstr(_)
				) {
					flag = true;
				}
			}
			if flag {
				for item in before_return.clone().into_iter() {
					block.borrow_mut().push(item);
				}
			}
		}

		let hasher = Rc::new(RefCell::new(hasher));
		func.cfg.blocks.push(hasher.clone());
		let length = func.cfg.blocks.len();
		func.cfg.blocks.swap(0, length - 1);

		let middles: Vec<_> =
			middles.into_iter().map(|item| Rc::new(RefCell::new(item))).collect();

		for item in middles.iter() {
			func.cfg.blocks.push(item.clone())
		}

		let go_to_normal = Rc::new(RefCell::new(go_to_normal));
		let success_return = Rc::new(RefCell::new(success_return));

		func.cfg.blocks.push(go_to_normal.clone());
		func.cfg.blocks.push(success_return.clone());

		hasher.borrow_mut().succ = vec![middles[0].clone()];

		for i in 1..utils::CACHE_SIZE {
			middles[i - 1].borrow_mut().succ =
				vec![middles[i].clone(), success_return.clone()];
		}

		success_return.borrow_mut().succ = vec![];

		middles[utils::CACHE_SIZE - 1].borrow_mut().succ =
			vec![success_return.clone(), go_to_normal.clone()];

		go_to_normal.borrow_mut().succ = vec![entry.clone()];

		func.cfg.analysis();
		// todo!()
	}
}
