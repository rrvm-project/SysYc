use std::{collections::HashMap, hash::Hash, iter::once};

use ast::tree::*;
use pest::{iterators::Pair, pratt_parser::PrattParser, Parser};
use pest_derive::Parser;
use utils::{errors::Result, SysycError::LexError};
use value::{BType, BinaryOp, FuncRetType, UnaryOp};

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
		Rule::LOr => BinaryOp::LOr,
		Rule::LAnd => BinaryOp::LAnd,
		_ => unreachable!(),
	}
}

fn map_unary_op(pair: &Pair<Rule>) -> UnaryOp {
	match pair.as_rule() {
		Rule::UnaryAdd => UnaryOp::Plus,
		Rule::UnarySub => UnaryOp::Neg,
		Rule::UnaryNot => UnaryOp::Not,
		Rule::UnaryBitNot => UnaryOp::BitNot,
		_ => unreachable!(),
	}
}

fn parse_identifier(pair: Pair<Rule>) -> String {
	match pair.as_rule() {
		Rule::Identifier => String::from(pair.as_str()),
		_ => unreachable!(),
	}
}

fn parse_var_type(pair: Pair<Rule>) -> BType {
	match pair.as_rule() {
		Rule::int_t => BType::Int,
		Rule::float_t => BType::Float,
		_ => unreachable!(),
	}
}

fn parse_func_type(pair: Pair<Rule>) -> FuncRetType {
	match pair.as_rule() {
		Rule::int_t => FuncRetType::Int,
		Rule::float_t => FuncRetType::Float,
		Rule::void_t => FuncRetType::Void,
		_ => unreachable!(),
	}
}

fn parse_dim_list(pair: Pair<Rule>) -> NodeList {
	pair.into_inner().map(parse_expr).collect()
}

lazy_static::lazy_static! {
	static ref PRATT_PARSER: PrattParser<Rule> = {
		use pest::pratt_parser::{Assoc::*, Op};
		PrattParser::new()
			.op(Op::infix(Rule::Assign, Right))
			.op(Op::infix(Rule::LOr, Left))
			.op(Op::infix(Rule::LAnd, Left))
			.op(Op::infix(Rule::EQ, Left) | Op::infix(Rule::NE, Left))
			.op(Op::infix(Rule::LE, Left) | Op::infix(Rule::LT, Left)
				| Op::infix(Rule::GE, Left) | Op::infix(Rule::GT, Left))
			.op(Op::infix(Rule::Add, Left) | Op::infix(Rule::Sub, Left))
			.op(Op::infix(Rule::Mul, Left) | Op::infix(Rule::Div, Left) | Op::infix(Rule::Mod, Left))
			.op(Op::prefix(Rule::UnaryAdd) | Op::prefix(Rule::UnarySub)
				| Op::prefix(Rule::UnaryNot) | Op::prefix(Rule::UnaryBitNot))
	};
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
	let mut var: Node = Box::new(Variable {
		_attrs: HashMap::new(),
		ident: parse_identifier(pairs.next().unwrap()),
	});
	let pairs = pairs.next().unwrap().into_inner();
	for v in pairs {
		var = Box::new(BinaryExpr {
			_attrs: HashMap::new(),
			lhs: var,
			op: BinaryOp::IDX,
			rhs: parse_expr(v),
		});
	}
	var
}

fn parse_float_lit(s: &str) -> f32 {
	let mut s1 = s.to_string();
	if s1.starts_with('.') {
		s1.insert(0, '0');
	}
	if s1.ends_with('f') {
		s1.pop();
	}
	s1.parse().unwrap()
}

fn parse_int_lit(s: &str) -> i32 {
	if s == "0" {
		return 0;
	}
	if s.contains('x') || s.contains('X') {
		return i32::from_str_radix(&s[2..], 16).unwrap();
	} else if s.contains('b') || s.contains('B') {
		return i32::from_str_radix(&s[2..], 2).unwrap();
	}
	if s.starts_with('0') {
		return i32::from_str_radix(s, 8).unwrap();
	}
	s.parse().unwrap()
}

fn parse_primary_expr(pair: Pair<Rule>) -> Node {
	match pair.as_rule() {
		Rule::Integer => Box::new(LiteralInt {
			_attrs: HashMap::new(),
			value: parse_int_lit(pair.as_str()),
		}),

		Rule::Float => Box::new(LiteralFloat {
			_attrs: HashMap::new(),
			value: parse_float_lit(pair.as_str()),
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
		.map_primary(parse_primary_expr)
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
		val_list: pair.into_inner().map(parse_init_val).collect(),
	};
	Box::new(init_val_list)
}

fn parse_var_def(pair: Pair<Rule>) -> Node {
	let mut pairs = pair.into_inner();
	let mut var_def = VarDef {
		_attrs: HashMap::new(),
		ident: parse_identifier(pairs.next().unwrap()),
		dim_list: Vec::new(),
		init: None,
	};
	for pair in pairs {
		match pair.as_rule() {
			Rule::DimList => var_def.dim_list = parse_dim_list(pair),
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
			dim_list: pairs
				.next()
				.map(|v| once(LiteralInt::node(0)).chain(parse_dim_list(v)).collect())
				.unwrap_or_default(),
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
	let pair = pair.into_inner().next();
	fn parse_unwrap_stmt(pair: Pair<Rule>) -> Node {
		match pair.as_rule() {
			Rule::Expr => parse_expr(pair),
			Rule::Block => parse_block(pair),
			Rule::IfStmt => parse_if_stmt(pair),
			Rule::WhileStmt => parse_while_stmt(pair),
			Rule::Break => Box::<Break>::default(),
			Rule::Continue => Box::<Continue>::default(),
			Rule::Return => parse_return(pair),
			_ => unreachable!(),
		}
	}
	match pair {
		Some(pair) => parse_unwrap_stmt(pair),
		None => Box::new(Block {
			_attrs: HashMap::new(),
			stmts: Vec::new(),
		}),
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
		ret_type: parse_func_type(pairs.next().unwrap()),
		ident: parse_identifier(pairs.next().unwrap()),
		formal_params: parse_formal_params(pairs.next().unwrap()),
		block: parse_block(pairs.next().unwrap()),
	};
	Box::new(func_decl)
}

fn parse_comp_unit(pair: Pair<Rule>, program: &mut Program) {
	match pair.as_rule() {
		Rule::Decl => program.global_vars.push(parse_decl(pair)),
		Rule::FuncDecl => program.functions.push(parse_func_decl(pair)),
		Rule::EOI => (),
		_ => unreachable!(),
	}
}

pub fn parse(str: &str) -> Result<Program> {
	let pairs = SysycParser::parse(Rule::Program, str)
		.map_err(|e| LexError(e.to_string()))?;
	let mut program = Program {
		_attrs: HashMap::new(),
		global_vars: Vec::new(),
		functions: Vec::new(),
		next_temp: 0,
	};
	for pair in pairs.into_iter() {
		parse_comp_unit(pair, &mut program);
	}
	Ok(program)
}
