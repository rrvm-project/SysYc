use std::{collections::HashMap, hash::Hash};

use ast::{tree::*, BinaryOp, FuncType, UnaryOp, VarType};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use utils::SysycError;

#[derive(Parser)]
#[grammar = "sysy2022.pest"]
struct SysycParser;

fn map_binary_op(pair: &Pair<Rule>) -> BinaryOp {
	match pair.as_rule() {
		Rule::Assign => BinaryOp::Assign,
		Rule::Add => BinaryOp::Add,
		Rule::Sub => BinaryOp::Sub,
		Rule::Mul => BinaryOp::Mul,
		Rule::Div => BinaryOp::Div,
		Rule::Mod => BinaryOp::Mod,
		Rule::LT => BinaryOp::LT,
		Rule::LE => BinaryOp::LE,
		Rule::GE => BinaryOp::GE,
		Rule::GT => BinaryOp::GT,
		Rule::EQ => BinaryOp::EQ,
		Rule::NE => BinaryOp::NE,
		_ => unreachable!(),
	}
}

fn map_unary_op(pair: &Pair<Rule>) -> UnaryOp {
	match pair.as_rule() {
		Rule::UnaryAdd => UnaryOp::Plus,
		Rule::UnarySub => UnaryOp::Neg,
		Rule::UnaryNot => UnaryOp::Not,
		_ => unreachable!(),
	}
}

// TODO: 把这个改成 &str
fn parse_identifier(pair: Pair<Rule>) -> String {
	match pair.as_rule() {
		Rule::Identifier => String::from(pair.as_str()),
		_ => unreachable!(),
	}
}

fn parse_var_type(pair: Pair<Rule>) -> VarType {
	match pair.as_rule() {
		Rule::int_t => VarType::Int,
		Rule::float_t => VarType::Float,
		_ => unreachable!(),
	}
}

fn parse_func_type(pair: Pair<Rule>) -> FuncType {
	match pair.as_rule() {
		Rule::int_t => FuncType::Int,
		Rule::float_t => FuncType::Float,
		Rule::void_t => FuncType::Void,
		_ => unreachable!(),
	}
}

fn parse_dim_list(pair: Pair<Rule>, min_dims: usize) -> Option<NodeList> {
	let dim_list: NodeList = pair.into_inner().map(parse_expr).collect();
	if dim_list.len() >= min_dims {
		Some(dim_list)
	} else {
		None
	}
}

fn parse_func_call(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let func_call = FuncCall {
		_attrs: HashMap::new(),
		ident: parse_identifier(pairs.next().unwrap()),
		params: pairs.map(parse_expr).collect(),
	};
	Box::new(func_call)
}

fn parse_lval(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let lval = Lval {
		_attrs: HashMap::new(),
		ident: parse_identifier(pairs.next().unwrap()),
		dim_list: parse_dim_list(pairs.next().unwrap(), 1),
	};
	Box::new(lval)
}

fn parse_primary_expr(pair: Pair<Rule>) -> Node {
	match pair.as_rule() {
		Rule::Integer => Box::new(LiteralInt {
			_attrs: HashMap::new(),
			value: pair.as_str().parse().unwrap(),
		}),
		Rule::Float => Box::new(LiteralFloat {
			_attrs: HashMap::new(),
			value: pair.as_str().parse().unwrap(),
		}),
		Rule::FuncCall => parse_func_call(pair),
		Rule::Lval => parse_lval(pair),
		Rule::Expr => parse_expr(pair),
		_ => unreachable!(),
	}
}

fn parse_unary_expr(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let x = pairs.next().unwrap();
	if x.as_rule() == Rule::Primary {
		parse_primary_expr(x.into_inner().next().unwrap())
	} else {
		let rhs = pairs.next().unwrap();
		Box::new(UnaryExpr {
			_attrs: HashMap::new(),
			op: map_unary_op(&x),
			rhs: parse_unary_expr(rhs),
		})
	}
}

fn parse_expr(pair: Pair<Rule>) -> Node {
	if pair.as_rule() == Rule::UnaryExpr {
		return parse_unary_expr(pair);
	}
	let mut pairs = pair.into_inner();
	let lhs = pairs.next();
	let op = pairs.next();
	let rhs = pairs.next();
	if let Some(op) = op {
		Box::new(BinaryExpr {
			_attrs: HashMap::new(),
			lhs: parse_expr(lhs.unwrap()),
			op: map_binary_op(&op),
			rhs: parse_expr(rhs.unwrap()),
		})
	} else {
		parse_expr(lhs.unwrap())
	}
}

fn parse_init_val(pair: Pair<Rule>) -> Node {
	match pair.as_rule() {
		Rule::Expr => parse_expr(pair),
		Rule::InitValList => parse_init_val_list(pair),
		_ => unreachable!(),
	}
}

fn parse_init_val_list(pair: Pair<Rule>) -> Node {
	let init_val_list = InitValList {
		_attrs: HashMap::new(),
		val_list: pair.into_inner().map(parse_init_val).collect(),
	};
	Box::new(init_val_list)
}

fn parse_var_def(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let mut var_def = VarDef {
		_attrs: HashMap::new(),
		ident: parse_identifier(pairs.next().unwrap()),
		dim_list: None,
		init: None,
	};
	for pair in pairs {
		match pair.as_rule() {
			Rule::DimList => var_def.dim_list = parse_dim_list(pair, 1),
			Rule::Expr => var_def.init = Some(parse_init_val(pair)),
			Rule::InitValList => var_def.init = Some(parse_init_val(pair)),
			_ => unreachable!(),
		}
	}
	Box::new(var_def)
}

fn parse_decl(pair: Pair<Rule>) -> Node {
	let pair = pair.into_inner().next().unwrap();
	fn _parse(pair: Pair<Rule>, is_const: bool) -> VarDecl {
		let mut pairs = pair.into_inner();
		VarDecl {
			_attrs: HashMap::new(),
			is_const,
			type_t: parse_var_type(pairs.next().unwrap()),
			defs: pairs.map(parse_var_def).collect(),
		}
	}
	match pair.as_rule() {
		Rule::ConstDecl => Box::new(_parse(pair, true)),
		Rule::VarDecl => Box::new(_parse(pair, false)),
		_ => unreachable!(),
	}
}

fn parse_formal_params(pair: Pair<Rule>) -> NodeList {
	fn parse_formal_param(pair: Pair<Rule>) -> Node {
		let mut pairs = pair.into_inner();
		let formal_param = FormalParam {
			_attrs: HashMap::new(),
			type_t: parse_var_type(pairs.next().unwrap()),
			ident: parse_identifier(pairs.next().unwrap()),
			dim_list: pairs.next().and_then(|v| parse_dim_list(v, 0)),
		};
		Box::new(formal_param)
	}
	pair.into_inner().map(parse_formal_param).collect()
}

fn parse_if_stmt(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let if_stmt = If {
		_attrs: HashMap::new(),
		cond: parse_expr(pairs.next().unwrap()),
		body: parse_stmt(pairs.next().unwrap()),
		then: pairs.next().map(parse_stmt),
	};
	Box::new(if_stmt)
}

fn parse_while_stmt(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let while_stmt = While {
		_attrs: HashMap::new(),
		cond: parse_expr(pairs.next().unwrap()),
		body: parse_stmt(pairs.next().unwrap()),
	};
	Box::new(while_stmt)
}

fn parse_return(pair: Pair<Rule>) -> Node {
	let return_stmt = Return {
		_attrs: HashMap::new(),
		value: pair.into_inner().next().map(parse_expr),
	};
	Box::new(return_stmt)
}

fn parse_stmt(pair: Pair<Rule>) -> Node {
	let pair = pair.into_inner().next().unwrap();
	match pair.as_rule() {
		Rule::Expr => parse_expr(pair),
		Rule::Block => parse_block(pair),
		Rule::IfStmt => parse_if_stmt(pair),
		Rule::WhileStmt => parse_while_stmt(pair),
		Rule::Break => Box::new(Break::new()),
		Rule::Continue => Box::new(Continue::new()),
		Rule::Return => parse_return(pair),
		_ => unreachable!(),
	}
}

fn parse_block(pair: Pair<Rule>) -> Node {
	fn parse_block_item(pair: Pair<Rule>) -> Node {
		match pair.as_rule() {
			Rule::Decl => parse_decl(pair),
			Rule::Stmt => parse_stmt(pair),
			_ => unreachable!(),
		}
	}
	let block = Block {
		_attrs: HashMap::new(),
		stmts: pair.into_inner().map(parse_block_item).collect(),
	};
	Box::new(block)
}

fn parse_func_decl(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let func_decl = FuncDecl {
		_attrs: HashMap::new(),
		func_type: parse_func_type(pairs.next().unwrap()),
		ident: parse_identifier(pairs.next().unwrap()),
		formal_params: parse_formal_params(pairs.next().unwrap()),
		block: parse_block(pairs.next().unwrap()),
	};
	Box::new(func_decl)
}

fn parse_comp_unit(pair: Pair<Rule>) -> Option<Node> {
	match pair.as_rule() {
		Rule::Decl => Some(parse_decl(pair)),
		Rule::FuncDecl => Some(parse_func_decl(pair)),
		Rule::EOI => None,
		_ => unreachable!(),
	}
}

#[allow(unused)]
pub fn parse(str: &str) -> Result<Program, SysycError> {
	let progam = SysycParser::parse(Rule::Program, str)
		.map_err(|e| SysycError::DecafLexError(e.to_string()))?;
	Ok(Program {
		_attrs: HashMap::new(),
		comp_units: progam.into_iter().filter_map(parse_comp_unit).collect(),
	})
}
