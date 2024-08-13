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

		calc_var_data(program, metadata);
		calc_func_data(program, metadata);

		program
			.global_vars
			.retain(|v| metadata.var_data.contains_key(&(v.ident.clone(), 0)));

		eprintln!("{:#?}", metadata.var_data);

		Ok(false)
	}
}
