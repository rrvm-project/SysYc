use std::{collections::HashMap, hash::Hash};

use ast::{tree::*, BinaryOp, Type, UnaryOp};
use pest::{iterators::Pair, pratt_parser::PrattParser, Parser};
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
		Rule::LQ => BinaryOp::LQ,
		Rule::LE => BinaryOp::LE,
		Rule::GE => BinaryOp::GE,
		Rule::GQ => BinaryOp::GQ,
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

// TODO: 我摆了，太多东西了不想填生命周期了，谁闲的话把这个改成 &str
fn parse_identifier(pair: Pair<Rule>) -> String {
	match pair.as_rule() {
		Rule::Identifier => String::from(pair.as_str()),
		_ => unreachable!(),
	}
}

fn parse_type(pair: Pair<Rule>) -> Type {
	match pair.as_rule() {
		Rule::int_t => Type::Int,
		Rule::float_t => Type::Float,
		_ => unreachable!(),
	}
}

fn parse_dim_list(pair: Pair<Rule>, min_dims: usize) -> Option<Node> {
	let dim_list = DimList {
		_attrs: HashMap::new(),
		exprs: pair.into_inner().map(|v| parse_expr(v)).collect(),
	};
	if dim_list.exprs.len() >= min_dims {
		Some(Box::new(dim_list))
	} else {
		None
	}
}

lazy_static::lazy_static! {
	static ref PRATT_PARSER: PrattParser<Rule> = {
		use pest::pratt_parser::{Assoc::*, Op};
		PrattParser::new()
			.op(Op::infix(Rule::Assign, Left))
			.op(Op::infix(Rule::LOr, Left))
			.op(Op::infix(Rule::LAnd, Left))
			.op(Op::infix(Rule::EQ, Left) | Op::infix(Rule::NE, Left))
			.op(Op::infix(Rule::LQ, Left) | Op::infix(Rule::LE, Left)
				| Op::infix(Rule::GE, Left) | Op::infix(Rule::GQ, Left))
			.op(Op::infix(Rule::Add, Left) | Op::infix(Rule::Sub, Left))
			.op(Op::infix(Rule::Mul, Left) | Op::infix(Rule::Div, Left) | Op::infix(Rule::Mod, Left))
			.op(Op::prefix(Rule::UnaryAdd) | Op::prefix(Rule::UnarySub) | Op::prefix(Rule::UnaryNot))
	};
}

fn parse_func_call(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let func_call = FuncCall {
		_attrs: HashMap::new(),
		ident: parse_identifier(pairs.next().unwrap()),
		params: pairs.map(|v| parse_expr(v)).collect(),
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

fn parse_binary_expr(lhs: Node, op: Pair<Rule>, rhs: Node) -> Node {
	Box::new(BinaryExpr {
		_attrs: HashMap::new(),
		lhs,
		op: map_binary_op(&op),
		rhs,
	})
}

fn parse_unary_expr(op: Pair<Rule>, rhs: Node) -> Node {
	Box::new(UnaryExpr {
		_attrs: HashMap::new(),
		op: map_unary_op(&op),
		rhs,
	})
}

fn parse_expr(pair: Pair<Rule>) -> Node {
	PRATT_PARSER
		.map_primary(|v| parse_primary_expr(v))
		.map_infix(|lhs, op, rhs| parse_binary_expr(lhs, op, rhs))
		.map_prefix(|op, rhs| parse_unary_expr(op, rhs))
		.parse(pair.into_inner())
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
		val_list: pair.into_inner().map(|v| parse_init_val(v)).collect(),
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
	for pair in pairs.into_iter() {
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
	fn _parse(pair: Pair<Rule>, is_const: bool) -> VarDecl {
		let mut pairs = pair.into_inner();
		VarDecl {
			_attrs: HashMap::new(),
			is_const,
			type_t: parse_type(pairs.next().unwrap()),
			defs: pairs.into_iter().map(|v| parse_var_def(v)).collect(),
		}
	}
	match pair.as_rule() {
		Rule::ConstDecl => Box::new(_parse(pair, true)),
		Rule::VarDecl => Box::new(_parse(pair, false)),
		_ => unreachable!(),
	}
}

#[allow(unused)]
fn parse_func_decl(pair: Pair<Rule>) -> Node {
	todo!()
}

fn parse_comp_unit(pair: Pair<Rule>) -> Option<Node> {
	match pair.as_rule() {
		Rule::Decl => Some(parse_decl(pair.into_inner().next().unwrap())),
		Rule::FuncDecl => Some(parse_func_decl(pair.into_inner().next().unwrap())),
		Rule::EOI => None,
		_ => unreachable!(),
	}
}

pub fn parse(str: &str) -> Result<Program, SysycError> {
	let progam = SysycParser::parse(Rule::Program, str)
		.map_err(|e| SysycError::DecafLexError(e.to_string()))?;
	Ok(Program {
		_attrs: HashMap::new(),
		comp_units: progam
			.into_iter()
			.map(|v| parse_comp_unit(v))
			.filter_map(|x| x)
			.collect(),
	})
}
