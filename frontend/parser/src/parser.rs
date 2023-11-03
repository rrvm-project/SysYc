use std::{collections::HashMap, hash::Hash};

use ast::{tree::*, Type};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use utils::SysycError;

#[derive(Parser)]
#[grammar = "sysy2022.pest"]
struct SysycParser;

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

#[allow(unused)]
fn parse_dim_list(pair: Pair<Rule>) -> Node {
	todo!()
}

#[allow(unused)]
fn parse_expr(pair: Pair<Rule>) -> Node {
	todo!()
}

fn parse_init_val(pair: Pair<Rule>) -> Node {
	match pair.as_rule() {
		Rule::Expr => parse_expr(pair),
		Rule::InitValList => parse_init_val_list(pair),
		_ => unreachable!()
	}
}

fn parse_init_val_list(pair: Pair<Rule>) -> Node {
	let init_val_list = InitValList {
		_attrs: HashMap::new(),
		val_list: pair.into_inner().map(|v| parse_init_val(v)).collect()
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
			Rule::DimList => var_def.dim_list = Some(parse_dim_list(pair)),
			Rule::Expr => var_def.init = Some(parse_init_val(pair)),
			Rule::InitValList => var_def.init = Some(parse_init_val(pair)),
			_ => unreachable!()
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

fn parse_comp_unit(pair: Pair<Rule>) -> Node {
	match pair.as_rule() {
		Rule::Decl => parse_decl(pair.into_inner().next().unwrap()),
		Rule::FuncDecl => parse_func_decl(pair.into_inner().next().unwrap()),
		_ => unreachable!(),
	}
}

pub fn parse(str: &str) -> Result<Program, SysycError> {
	let progam = SysycParser::parse(Rule::Program, str)
		.map_err(|e| SysycError::DecafLexError(e.to_string()))?;
	Ok(Program {
		_attrs: HashMap::new(),
		comp_units: progam.into_iter().map(|v| parse_comp_unit(v)).collect(),
	})
}
