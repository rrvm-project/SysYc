use pest::Parser;
use pest_derive::Parser;
use utils::{errors::Result, SysycError::LlvmSyntexError};

#[derive(Parser)]
#[grammar = "llvmir.pest"]
struct IrParser;

pub fn parse(str: &str) -> Result<()> {
	let _progam = IrParser::parse(Rule::Program, str)
		.map_err(|e| LlvmSyntexError(e.to_string()))?;
	todo!()
}
