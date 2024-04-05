mod dead_code;
mod fold_constants;
mod function_inline;
mod fuyuki_vn;
pub mod impls;
mod loops;
mod pure_check;
mod strength_reduce;
mod tail_recursion;
mod unreachable;
mod useless_code;
mod useless_phis;
use std::{
	collections::HashSet,
	sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use rrvm::program::LlvmProgram;
use utils::errors::Result;

pub trait RrvmOptimizer {
	fn new() -> Self
	where
		Self: Sized;
	fn apply(&self, program: &mut LlvmProgram) -> Result<bool>;
}

#[derive(Default)]
pub struct Optimizer0 {}
#[derive(Default)]
pub struct Optimizer1 {}
#[derive(Default)]
pub struct Optimizer2 {}

lazy_static! {
	// 记录各个优化级别中被忽略的优化 pass
	pub static ref O0_IGNORE: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
	pub static ref O1_IGNORE: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
	pub static ref O2_IGNORE: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
}

#[allow(non_snake_case)]
pub fn global_ignore_O0(pass: &str) {
	O0_IGNORE.lock().unwrap().insert(pass.to_string());
}

#[allow(non_snake_case)]
pub fn global_ignore_O1(pass: &str) {
	O1_IGNORE.lock().unwrap().insert(pass.to_string());
}

#[allow(non_snake_case)]
pub fn global_ignore_O2(pass: &str) {
	O2_IGNORE.lock().unwrap().insert(pass.to_string());
}
