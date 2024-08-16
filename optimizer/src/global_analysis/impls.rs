use rrvm::program::LlvmProgram;

use crate::{metadata::MetaData, RrvmOptimizer};

use super::{
	func_data::calc_func_data, var_data::calc_var_data, GlobalAnalysis,
};

use utils::Result;

impl RrvmOptimizer for GlobalAnalysis {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		metadata.var_data.clear();
		metadata.get_var_data(&("getarray".to_owned(), 0)).to_store = true;
		metadata.get_var_data(&("getfarray".to_owned(), 0)).to_store = true;
		metadata.get_var_data(&("putarray".to_owned(), 1)).to_load = true;
		metadata.get_var_data(&("putfarray".to_owned(), 1)).to_load = true;
		metadata.get_var_data(&("putf".to_owned(), 0)).to_load = true;

		metadata.get_func_data("getint").set_syscall();
		metadata.get_func_data("getch").set_syscall();
		metadata.get_func_data("getfloat").set_syscall();
		metadata.get_func_data("getarray").set_syscall();
		metadata.get_func_data("getfarray").set_syscall();

		metadata.get_func_data("putint").set_syscall();
		metadata.get_func_data("putch").set_syscall();
		metadata.get_func_data("putfloat").set_syscall();
		metadata.get_func_data("putarray").set_syscall();
		metadata.get_func_data("putfarray").set_syscall();

		metadata.get_func_data("putf").set_syscall();
		metadata.get_func_data("starttime").set_syscall();
		metadata.get_func_data("stoptime").set_syscall();

		calc_var_data(program, metadata);
		calc_func_data(program, metadata);

		program
			.global_vars
			.retain(|v| metadata.var_data.contains_key(&(v.ident.clone(), 0)));

		metadata.func_data.values_mut().for_each(|v| {
			v.pure =
				v.usage_info.may_loads.is_empty() && v.usage_info.may_stores.is_empty()
		});
		for ((name, _), info) in metadata.var_data.iter() {
			if info.to_load || info.to_store {
				if let Some(func_data) = metadata.func_data.get_mut(name) {
					func_data.pure = false;
				}
			}
		}

		Ok(false)
	}
}
