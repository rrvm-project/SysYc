use std::fmt::Display;

use llvm::{CompOp, LlvmTemp, Value};

use super::LlvmNode;

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

/*
设循环为：

  int i = 0;
  while(i < 10){
	i = i + 1;
  }
  return i;

  entry:
    br label %B2
  B2:
    %1 = phi i32 [0, label %entry], [%3, label %B4]
    %2 = icmp slt i32 %1, 10
    %3 = add i32 %1, 1
    br i32 %2, label %B4, label %B8
  B4:
    br label %B2
  B8:
    ret i32 %1
*/
pub struct SimpleLoopInfo {
	pub loop_type: LoopType,
	// 0
	pub start: Option<Value>,
	// 10
	pub end: Option<Value>,
	// 1
	pub step: Option<Value>,
	// %2, 即用于 branch 指令的变量
	pub cond_temp: Option<LlvmTemp>,
	// %3, 归纳变量，其实就是每次循环都递增的那个控制循环是否继续进行的循环变量，is_simple_induction_variable() 中的第二个返回值
	pub indvar_temp: Option<LlvmTemp>,
	// %1, phi 指令的结果
	pub phi_temp: Option<LlvmTemp>,
	// slt
	pub cond_op: CompOp,
	// 循环体内指令数, Call 指令被认为是 50 条指令
	pub instr_cnt: i64, 
	// B2, 循环出口的前驱
	pub exit_prev: Option<LlvmNode>,
	// B8, 循环出口
	pub exit: Option<LlvmNode>,
	// entry， 从循环外进入循环入口的基本块
	pub into_entry: Option<LlvmNode>,
	// B4, 回边的起点
	pub backedge_start: Option<LlvmNode>,
}

impl SimpleLoopInfo {
	pub fn new() -> Self {
		Self {
			loop_type: LoopType::IGNORE,
			start: None,
			end: None,
			step: None,
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
		s.push_str(&self.start.clone().map_or(format!("start: None"), |s| format!("start: {}\n", s)));
		s.push_str(&self.end.clone().map_or(format!("end: None"), |s| format!("end: {}\n", s)));
		s.push_str(&self.step.clone().map_or(format!("step: None"), |s| format!("step: {}\n", s)));
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
