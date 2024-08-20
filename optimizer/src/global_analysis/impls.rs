use rrvm::program::LlvmProgram;

use crate::{metadata::MetaData, RrvmOptimizer};

use super::{
	func_data::calc_func_data, var_data::calc_var_data, GlobalAnalysis,
};

use utils::Result;

pub const BUILTIN_FUNCS: &[&str] = &[
	"getint",
	"getch",
	"getfloat",
	"getarray",
	"getfarray",
	"putint",
	"putch",
	"putfloat",
	"putarray",
	"putfarray",
	"putf",
	"_sysy_starttime",
	"_sysy_stoptime",
	"__create_threads",
	"__join_threads",
	"__fill_zero_words",
];

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

		for func in BUILTIN_FUNCS {
			metadata.get_func_data(func).set_syscall();
		}

		calc_var_data(program, metadata);
		calc_func_data(program, metadata);

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
