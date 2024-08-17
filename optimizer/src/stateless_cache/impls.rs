use llvm::{
	LlvmInstrTrait, LlvmInstrVariant, LlvmTemp, LlvmTempManager, StoreInstr,
	VarType,
};

use rrvm::program::LlvmFunc;
use utils::{
	GlobalVar,
	ValueItem::{Word, Zero},
};

use crate::RrvmOptimizer;

use super::StatelessCache;
fn check_type(var_type: VarType) -> Option<VarType> {
	match var_type {
		VarType::I32 => Some(VarType::I32),
		VarType::F32 => Some(VarType::F32),
		_ => None,
	}
}

fn check_para_return(func: &LlvmFunc) -> Option<(Vec<LlvmTemp>, VarType)> {
	let return_type = check_type(func.ret_type)?;

	let mut paras = vec![];
	for item in func.params.iter() {
		let tmp = item.unwrap_temp()?;
		paras.push(tmp);
	}

	Some((paras, return_type))
}

fn check_calls(func: &LlvmFunc) -> Option<usize> {
	let mut times = 0;

	for bb in func.cfg.blocks.iter() {
		for instr in bb.borrow().instrs.iter() {
			if let llvm::LlvmInstrVariant::CallInstr(call) = instr.get_variant() {
				if call.get_label().to_string() == func.name {
					times += 1;
				} else {
					return None;
				}
			}
		}
	}
	Some(times)
}

fn process_func(
	func: &mut LlvmFunc,
	mgr: &mut LlvmTempManager,
	global: &mut Vec<GlobalVar>,
) -> Option<()> {
	if check_calls(func)? <= 1 {
		return None;
	}
	let (params, return_type) = check_para_return(func)?;

	if params.is_empty() || params.len() > utils::CACHE_MAX_ARGS {
		return None;
	}

	let func_name = func.name.clone();

	let get_arg_hash_name =
		format!("{}_{}_ARG", utils::CACHE_PREFIX, func_name.as_str());

	let get_return_name =
		format!("{}_{}_RETURN", utils::CACHE_PREFIX, func_name.as_str());

	let get_begin_name =
		format!("{}_{}_BEGIN", utils::CACHE_PREFIX, func_name.as_str());

	let global_return = GlobalVar {
		ident: get_return_name,
		data: vec![Zero(utils::CACHE_SIZE * 4)], // return value is 4 Bytes wide
		is_float: return_type == VarType::F32,
	};
	global.push(global_return.clone());

	let global_hash = GlobalVar {
		ident: get_arg_hash_name,
		data: vec![Word(utils::CACHE_MAGIC); utils::CACHE_SIZE * 2], // Hash is 64 bit
		is_float: false,
	};
	global.push(global_hash.clone());

	let begin = GlobalVar {
		ident: get_begin_name,
		data: vec![Word(0)],
		is_float: false,
	};
	global.push(begin.clone());

	let store_addr = mgr.new_temp(return_type.to_ptr(), false);

	func.params.push(store_addr.clone().into());

	for block in func.cfg.blocks.iter_mut() {
		let mut return_value = None;
		if let Some(LlvmInstrVariant::RetInstr(r)) =
			block.borrow().jump_instr.as_ref().map(|i| i.get_variant())
		{
			return_value = r.value.clone();
		}

		if let Some(return_value) = return_value {
			let store = Box::new(StoreInstr {
				value: return_value,
				addr: store_addr.clone().into(),
			});

			block.borrow_mut().instrs.push(store);
		}

		func.need_cache = true;
	}

	Some(())
}
impl RrvmOptimizer for StatelessCache {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut rrvm::prelude::LlvmProgram,
		metadata: &mut crate::metadata::MetaData,
	) -> utils::Result<bool> {
		let mut changed = false;
		for func in program.funcs.iter_mut() {
			if !metadata.is_stateless(&func.name) {
				continue;
			}

			changed |=
				process_func(func, &mut program.temp_mgr, &mut program.global_vars)
					.is_some();
		}
		Ok(changed)
	}
}
