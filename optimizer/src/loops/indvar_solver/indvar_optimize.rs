// 包含外推和化简

use super::IndVarSolver;

impl<'a> IndVarSolver<'a> {
	// 确定每个变量是不是循环变量，如果是，确定每个循环变量有没有用
	pub fn classify_variant(&mut self) {
		let blocks =
			self.cur_loop.borrow().blocks_without_subloops(self.cfg, self.loop_map);
		for block in blocks {
			for inst in block.borrow().phi_instrs.iter() {
				if !self.visited.contains(&inst.target) {
					self.run(inst.target.clone());
				}
			}
			for inst in block.borrow().instrs.iter() {
				if let Some(t) = inst.get_write() {
					if !self.visited.contains(&t) {
						self.run(t);
					}
				}
			}
		}
	}
}
