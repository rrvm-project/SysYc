#![allow(unused)]

use crate::LlvmOp;
use pest::Parser;
use pest_derive::Parser;
use utils::SysycError;

#[derive(Parser)]
#[grammar = "llvmir.pest"]
struct IrParser;

pub fn parse(str: &str) -> Result<Vec<LlvmOp>, SysycError> {
	let progam = IrParser::parse(Rule::Program, str)
		.map_err(|e| SysycError::LlvmSyntexError(e.to_string()))?;
	todo!()
}
