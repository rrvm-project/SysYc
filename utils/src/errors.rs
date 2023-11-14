use thiserror::Error;

pub use SysycError::*;

#[derive(Error, Debug)]
pub enum SysycError {
	#[error("{0}")]
	DecafLexError(String),
	#[error("Syntax Error : {0}")]
	SyntaxError(String),
	#[error("System error: {0}")]
	SystemError(String),
	#[error("Llvm syntex error: {0}")]
	LlvmSyntexError(String),
	#[error("Riscv generating error: {0}")]
	RiscvGenError(String),
}

pub type Result<T, E = SysycError> = core::result::Result<T, E>;

pub fn map_sys_err(e: std::io::Error) -> SysycError {
	SystemError(e.to_string())
}
