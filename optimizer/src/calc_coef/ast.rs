use std::{cell::RefCell, collections::HashMap, rc::Rc};

use llvm::{ArithOp, CompOp, LlvmInstrTrait, LlvmTemp, Value, VarType};
use utils::Label;
#[derive(PartialEq, Eq,Clone)]
pub enum LlvmOp{
    ArithOp(ArithOp),
    CompOp(CompOp),
}
#[derive(PartialEq, Eq,Clone)]
pub enum AstNode {
	Value(Value),
	Expr((Rc<RefCell<AstNode>>, LlvmOp, Rc<RefCell<AstNode>>)),
	CallVal(String, Vec<Rc<RefCell<AstNode>>>),
	PhiNode(Vec<(Rc<RefCell<AstNode>>, Label)>),
}
pub enum ReduceType{
	Sub,
	Half,
}
pub fn get_ast_node(val:&Value,ast_map:&HashMap<LlvmTemp,Rc<RefCell<AstNode>>>)->Rc<RefCell<AstNode>>{
	if let Value::Temp(t) = val{
		if let Some(ast_node) = ast_map.get(t){
			ast_node.clone()
		}else{
			unreachable!();
		}
	}else{
		Rc::new(RefCell::new(AstNode::Value(val.clone())))
	}
}
pub fn map_ast_instr(instr: &Box<dyn LlvmInstrTrait>,ast_map:&mut HashMap<LlvmTemp,Rc<RefCell<AstNode>>>){
	use llvm::LlvmInstrVariant::*;
	match instr.get_variant(){
		ArithInstr(arith_instr)=>{
			let rs1=get_ast_node(&arith_instr.lhs,ast_map);
			let rs2=get_ast_node(&arith_instr.rhs,ast_map);
			let res=Rc::new(RefCell::new(AstNode::Expr((rs1, LlvmOp::ArithOp(arith_instr.op), rs2))));
			ast_map.insert(arith_instr.target.clone(),res);
		}
		CompInstr(comp_instr)=>{
			let rs1=get_ast_node(&comp_instr.lhs,ast_map);
			let rs2=get_ast_node(&comp_instr.rhs,ast_map);
			let res=Rc::new(RefCell::new(AstNode::Expr((rs1, LlvmOp::CompOp(comp_instr.op), rs2))));
			ast_map.insert(comp_instr.target.clone(),res);
		}
		PhiInstr(phi_instr)=>{
			let mut phi_nodes=Vec::new();
			for (val,label) in &phi_instr.source{
				let ast_node=get_ast_node(val,ast_map);
				phi_nodes.push((ast_node,label.clone()));
			}
			let res=Rc::new(RefCell::new(AstNode::PhiNode(phi_nodes)));
			ast_map.insert(phi_instr.target.clone(),res);
		}
		CallInstr(call_instr)=>{
			let mut args=Vec::new();
			for (var_type,arg) in &call_instr.params{
				args.push(get_ast_node(arg,ast_map));
			}
			let res=Rc::new(RefCell::new(AstNode::CallVal(call_instr.func.to_string().clone(),args)));
			ast_map.insert(call_instr.target.clone(),res);
		}
		_=>{}
	}
}
