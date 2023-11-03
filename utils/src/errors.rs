use thiserror::Error;

#[derive(Error, Debug)]
pub enum SysycError {
	#[error("{0}")]
	DecafLexError(String),
	#[error("system error: {0}")]
	SystemError(String),
}

pub fn map_sys_err(e: std::io::Error) -> SysycError {
	SysycError::SystemError(e.to_string())
}
