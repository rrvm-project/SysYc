use std::fmt::Display;

use llvm::{CompOp, LlvmTemp};

use crate::LlvmNode;

pub mod get_loop_info;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
	IGNORE,          // loop 比较复杂，不处理
	CONSTTERMINATED, //i = c0, i </<= c1, i++; c1 是常数
	VARTEMINATED,    // i = c0, i </<= n, i++; n 是变量
}

impl Display for LoopType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			LoopType::IGNORE => "IGNORE",
			LoopType::CONSTTERMINATED => "CONSTTERMINATED",
			LoopType::VARTEMINATED => "VARTEMINATED",
		};
		write!(f, "{}", s)
	}
}

pub struct SimpleLoopInfo {
	pub loop_type: LoopType,
	pub start: i32,
	pub end: i32,
	pub step: i32,
	pub end_temp: Option<LlvmTemp>,
	pub new_end_temp: Option<LlvmTemp>,
	pub cond_temp: Option<LlvmTemp>,
	// 归纳变量，其实就是每次循环都递增的那个控制循环是否继续进行的循环变量，is_simple_induction_variable() 中的第二个返回值
	pub indvar_temp: Option<LlvmTemp>,
	pub phi_temp: Option<LlvmTemp>,
	pub cond_op: CompOp,
	pub instr_cnt: i64, // 循环体内指令数, Call 指令被认为是 50 条指令
	pub exit_prev: Option<LlvmNode>,
	pub exit: Option<LlvmNode>,
	// 从循环外进入循环 entry 的基本块
	pub into_entry: Option<LlvmNode>,
	// 回边的起点
	pub backedge_start: Option<LlvmNode>,
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
			phi_temp: None,
			cond_op: CompOp::EQ,
			instr_cnt: 0,
			exit_prev: None,
			exit: None,
			into_entry: None,
			backedge_start: None,
		}
	}
}

impl Default for SimpleLoopInfo {
	fn default() -> Self {
		Self::new()
	}
}

impl Display for SimpleLoopInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut s = String::new();
		s.push_str(&format!("loop_type: {}\n", self.loop_type));
		s.push_str(&format!("start: {}\n", self.start));
		s.push_str(&format!("end: {}\n", self.end));
		s.push_str(&format!("step: {}\n", self.step));
		s.push_str(
			&self.end_temp.as_ref().map_or("end_temp: None\n".to_string(), |v| {
				format!("end_temp: {}\n", v)
			}),
		);
		s.push_str(
			&self
				.new_end_temp
				.as_ref()
				.map_or("new_end_temp: None\n".to_string(), |v| {
					format!("new_end_temp: {}\n", v)
				}),
		);
		s.push_str(
			&self.cond_temp.as_ref().map_or("cond_temp: None\n".to_string(), |v| {
				format!("cond_temp: {}\n", v)
			}),
		);
		s.push_str(
			&self
				.indvar_temp
				.as_ref()
				.map_or("indvar_temp: None\n".to_string(), |v| {
					format!("indvar_temp: {}\n", v)
				}),
		);
		s.push_str(&format!("cond_op: {}\n", self.cond_op));
		s.push_str(&format!("instr_cnt: {}\n", self.instr_cnt));
		s.push_str(
			&self.exit_prev.as_ref().map_or("exit_prev: None\n".to_string(), |v| {
				format!("exit_prev: {}\n", v.borrow().id)
			}),
		);
		s.push_str(&self.exit.as_ref().map_or("exit: None\n".to_string(), |v| {
			format!("exit: {}\n", v.borrow().id)
		}));
		s.push_str(
			&self.into_entry.as_ref().map_or("into_entry: None\n".to_string(), |v| {
				format!("into_entry: {}\n", v.borrow().id)
			}),
		);
		write!(f, "{}", s)
	}
}
