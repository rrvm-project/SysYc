use super::ZeroInit;
use std::{mem::replace, vec};

use crate::RrvmOptimizer;
use llvm::{
	CallInstr, LlvmInstrTrait, LlvmInstrVariant, LlvmTemp, LlvmTempManager,
	Value, VarType,
};
use rrvm::program::LlvmProgram;
use utils::{errors::Result, Label};

fn work(
	instrs: Vec<Box<dyn LlvmInstrTrait>>,
	tmp: &mut LlvmTempManager,
) -> Vec<Box<dyn LlvmInstrTrait>> {
	let mut result: Vec<Box<dyn LlvmInstrTrait>> = vec![];
	let mut pending: Vec<Box<dyn LlvmInstrTrait>> = vec![];

	let mut instrs = instrs.into_iter();

	enum State {
		Init,
		PendingStore(LlvmTemp),
		PendingGEP(LlvmTemp),
	}

	let mut state = State::Init;

	fn next(
		state: &State,
		instr: Option<Box<dyn LlvmInstrTrait>>,
	) -> (State, bool, Option<Box<dyn LlvmInstrTrait>>) {
		if let Some(this) = instr {
			match &this.get_variant() {
				LlvmInstrVariant::StoreInstr(i) => {
					if !matches!(i.value, Value::Int(0) | Value::Float(0f32)) {
						return (State::Init, false, Some(this));
					}

					match state {
						State::Init => (
							State::PendingGEP(i.addr.clone().get_temp().unwrap()),
							true,
							Some(this),
						),
						State::PendingStore(t) => {
							let tmp = i.addr.clone().get_temp().unwrap();
							if tmp == *t {
								(
									State::PendingGEP(i.addr.clone().get_temp().unwrap()),
									true,
									Some(this),
								)
							} else {
								(State::Init, false, Some(this))
							}
						}
						State::PendingGEP(_) => (State::Init, false, Some(this)),
					}
				}
				LlvmInstrVariant::GEPInstr(i) => {
					if let (target, Value::Temp(origin), Value::Int(4)) =
						(&i.target, &i.addr, &i.offset)
					{
						match state {
							State::PendingGEP(pending) => {
								if *pending == *origin {
									(State::PendingStore(target.clone()), true, Some(this))
								} else {
									(State::Init, false, Some(this))
								}
							}
							_ => (State::Init, false, Some(this)),
						}
					} else {
						(State::Init, false, Some(this))
					}
				}
				_ => (State::Init, false, Some(this)),
			}
		} else {
			(State::Init, false, None)
		}
	}

	//    fn instr_format<T: Display>(v: T) -> String {
	//         format!("  {}", v)
	//     }

	loop {
		let (newstate, expected, this) = next(&state, instrs.next());
		// let d : Vec<_> = this.iter().map(instr_format).collect();
		// println!("{} {:?} {}", expected, d, pending.len());
		if !expected {
			//1024个以上再启用这个机制！
			if pending.len() >= 2047 {
				let begin_addr = match pending.first().unwrap().get_variant() {
					LlvmInstrVariant::StoreInstr(i) => i.addr.clone(),
					_ => unreachable!(),
				};
				let var_type = begin_addr.get_type();
				let end_addr = match pending.last().unwrap().get_variant() {
					LlvmInstrVariant::GEPInstr(i) => i.target.clone(),
					LlvmInstrVariant::StoreInstr(_) => tmp.new_temp(var_type, false),
					_ => unreachable!(),
				};

				result.push(Box::new(CallInstr {
					target: end_addr,
					var_type,
					func: Label::new("__fill_zero_words"),
					params: vec![
						(var_type, begin_addr),
						(VarType::I32, (((pending.len() + 1) / 2) as i32).into()),
					],
				}));

				pending.clear();
			} else {
				result.append(&mut pending);
			}
		}

		if let Some(this) = this {
			if expected {
				pending.push(this);
			} else {
				result.push(this);
			}
		} else {
			break;
		}

		state = newstate;
	}

	result
}

impl RrvmOptimizer for ZeroInit {
	fn new() -> Self {
		ZeroInit {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		program.funcs.iter_mut().for_each(|func| {
			func.cfg.blocks.iter_mut().for_each(|block| {
				let instrs = std::mem::take(&mut block.borrow_mut().instrs);
				let _ = replace(
					&mut block.borrow_mut().instrs,
					work(instrs, &mut program.temp_mgr),
				);
			})
		});

		Ok(false)
	}
}
