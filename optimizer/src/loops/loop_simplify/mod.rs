use super::loop_optimizer::LoopOptimizer;

pub mod impls;

pub struct LoopSimplify<'a, 'b> {
	pub opter: &'b mut LoopOptimizer<'a>,
}
