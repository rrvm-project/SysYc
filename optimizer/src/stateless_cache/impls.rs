use std::{cell::RefCell, collections::HashMap, rc::Rc};

use llvm::{CallInstr, LlvmInstrTrait, LlvmTemp, LlvmTempManager, VarType};
use rrvm::{func::RrvmFunc, prelude::LlvmBasicBlock, program::LlvmFunc};
use utils::{math::increment, GlobalVar, ValueItem::{Word, Zero}};

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




fn check_calls(func: &LlvmFunc) -> Option<usize>{
    let mut times = 0;

    for bb in func.cfg.blocks.iter(){
        for instr in bb.borrow().instrs.iter(){
            if let llvm::LlvmInstrVariant::CallInstr(call) = instr.get_variant(){
                if call.get_label().to_string() == func.name{
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
    if check_calls(func)? <=1 {
        return None;
    }
    let (params, return_type) = check_para_return(func)?;


    if params.len() == 0 || params.len() > utils::CACHE_MAX_ARGS {
        return None;
    }


    let func_name = func.name.clone();
    let get_arg_name = |arg: &LlvmTemp| {
        format!("{}_{}_ARG_{}", utils::CACHE_PREFIX, func_name.as_str(), arg.to_string())
    };
    let get_return_name = || {
        format!("{}_{}_RETURN", utils::CACHE_PREFIX, func_name.as_str())
    };

    let get_begin_name = || {
        format!("{}_{}_BEGIN", utils::CACHE_PREFIX, func_name.as_str())
    };
    let get_end_name = || {
        format!("{}_{}_END", utils::CACHE_PREFIX, func_name.as_str())
    };


    let global_return = GlobalVar{ident: get_begin_name(), data: vec![Zero(utils::CACHE_SIZE)], is_float : return_type == VarType::F32};
    
    global.push(global_return.clone());
    let mut global_params : HashMap<LlvmTemp, GlobalVar> = HashMap::new();

    for param in params.iter(){
        match check_type(param.var_type)?{
            VarType::I32 => {
                global_params.insert(param.clone(), GlobalVar{
                    ident: get_arg_name(param),
                    data: vec![Zero(utils::CACHE_SIZE)],
                    is_float: false,
                });
            },
            VarType::F32 => {
                global_params.insert(param.clone(), GlobalVar{
                    ident: get_arg_name(param),
                    data: vec![Zero(utils::CACHE_SIZE)],
                    is_float: true,
                });
            },
            _ => unreachable!()
        }
    }


    global.extend(global_params.values().cloned());

    
    let begin = GlobalVar{ident: get_begin_name(), data: vec![Word(0)], is_float : false};
    let end = GlobalVar{ident: get_end_name(), data: vec![Word(0)], is_float : false};

    global.push(begin.clone());
    global.push(end.clone());

    let new_entry =   Rc::new(RefCell::new(LlvmBasicBlock::new(increment(&mut func.total), weight)));
    


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
				process_func(func, &mut program.temp_mgr, &mut program.global_vars).is_some();
		}
		Ok(changed)
	}
}
