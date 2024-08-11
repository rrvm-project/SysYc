use super::loop_optimizer::LoopOptimizer;

mod impls;
mod one_loop_solver;

pub struct IndvarOptimize<'a: 'b, 'b> {
	opter: &'b mut LoopOptimizer<'a>,
}
