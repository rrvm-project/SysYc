use thiserror::Error;

#[derive(Error, Debug)]
pub enum SysycError {
	#[error("{0}")]
	DecafLexError(String),
	#[error("Namer Error")]
	NamerError(String),
	#[error("system error: {0}")]
	SystemError(String),
	#[error("llvm syntex error: {0}")]
	LlvmSyntexError(String),
}

pub fn map_sys_err(e: std::io::Error) -> SysycError {
	SysycError::SystemError(e.to_string())
}
