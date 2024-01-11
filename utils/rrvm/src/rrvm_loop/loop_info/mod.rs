use instruction::Temp;
use llvm::{CompInstr, CompOp, JumpCondInstr};

use crate::LlvmNode;

pub mod get_loop_info;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
	IGNORE,          // loop 比较复杂，不处理
	CONSTTERMINATED, //i = c0, i </<= c1, i++; c1 是常数
	VARTEMINATED,    // i = c0, i </<= n, i++; n 是变量
}

pub struct SimpleLoopInfo {
	pub loop_type: LoopType,
	pub start: i32,
	pub end: i32,
	pub step: i32,
	pub end_temp: Option<Temp>,
	pub new_end_temp: Option<Temp>,
	pub cond_temp: Option<Temp>,
	pub indvar_temp: Option<Temp>,
	pub cond_op: CompOp,
	pub into_cond: Option<CompInstr>,
	pub into_branch: Option<JumpCondInstr>,
	pub instr_cnt: i64, // 循环体内指令数, Call 指令被认为是 50 条指令
	pub exit_prev: Option<LlvmNode>,
	pub exit: Option<LlvmNode>,
	pub into_entry: Option<LlvmNode>,
	pub new_into_entry: Option<LlvmNode>,
}

impl SimpleLoopInfo {
	pub fn new() -> Self {
		Self {
			loop_type: LoopType::IGNORE,
			start: 0,
			end: 0,
			step: 0,
			end_temp: None,
			new_end_temp: None,
			cond_temp: None,
			indvar_temp: None,
			cond_op: CompOp::EQ,
			into_cond: None,
			into_branch: None,
			instr_cnt: 0,
			exit_prev: None,
			exit: None,
			into_entry: None,
			new_into_entry: None,
		}
	}
}

impl Default for SimpleLoopInfo {
	fn default() -> Self {
		Self::new()
	}
}
