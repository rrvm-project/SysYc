use std::fmt::Display;

use llvm::{CompInstr, CompOp, JumpCondInstr, LlvmTemp};

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
	pub indvar_temp: Option<LlvmTemp>,
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
		s.push_str(
			&self
				.new_into_entry
				.as_ref()
				.map_or("new_into_entry: None\n".to_string(), |v| {
					format!("new_into_entry: {}\n", v.borrow().id)
				}),
		);
		write!(f, "{}", s)
	}
}
