use crate::{riscvinstr::*, riscvop::*};
use std::fmt::Display;

impl Display for ArithInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{} {},{},{}", self.op, self.tar, self.lhs, self.rhs)
	}
}
impl Display for LabelInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:", self.label.name)
	}
}
impl Display for CompInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {},{},{}", self.op, self.tar, self.lhs, self.rhs)
	}
}
impl Display for BrCondInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {},{},{}", self.op, self.tar, self.lhs, self.rhs)
	}
}
impl Display for JmpInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "j {}", self.tar)
	}
}
impl Display for BeqzInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {},{}", self.op, self.tar, self.lhs)
	}
}
impl Display for AllocInstr {
	//栈上alloc等价于先降sp再全store成0
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "addi sp,sp,-{}\n", self.length * 4);
		for i in 0..self.length {
			write!(f, "sw zero,{}(sp)\n", Value::Imm(4 * i as i32));
		}
		Ok(())
	}
}
impl Display for StoreInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "sw {},{}(sp)", self.value, self.offset)
	}
}
impl Display for LoadInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "lw {},{}(sp)", self.target, self.offset)
	}
}
impl Display for CallInstr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for i in &self.params {
			match i.0{
                Value::Imm(imm)=>{write!(f,"li {},{}",i.1,i.0).unwrap();}
                Value::Float(flt)=>{
                    write!()
                }
            }
		}
		write!(f, "call {}", self.func.name)
	}
}
impl Display for MvInstr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"mv {},{}",self.dst,self.src)
    }
}
impl Display for LiInstr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"li {},{}",self.dst,self.src)
    }
}
impl Display for ConvertInstr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{} {},{}",self.op,self.dst,self.src)
    }
}
