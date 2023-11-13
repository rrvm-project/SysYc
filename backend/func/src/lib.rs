use basicblock::transform_basicblock;
use rrvmfunc::RrvmFunc;
use utils::SysycError;

pub mod rrvmfunc;

pub fn transform_riscv(mut func: RrvmFunc) -> Result<RrvmFunc, SysycError> {
	let blocks: Result<_, _> =
		func.cfg.basic_blocks.into_iter().map(transform_basicblock).collect();
	func.cfg.basic_blocks = blocks?;
	Ok(func)
}
