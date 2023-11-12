use basicblock::transform_basicblock;
use rrvmfunc::RrvmFunc;

pub mod rrvmfunc;

pub fn transform_riscv(mut func: RrvmFunc) -> RrvmFunc {
	func.cfg.basic_blocks =
		func.cfg.basic_blocks.into_iter().map(transform_basicblock).collect();
	func
}
