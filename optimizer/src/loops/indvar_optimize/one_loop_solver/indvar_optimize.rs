// 包含外推和化简

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	// 确定每个变量是不是循环变量，如果是，确定每个循环变量有没有用
	pub fn classify_variant(&mut self) {
		let blocks = self
			.cur_loop
			.borrow()
			.blocks_without_subloops(&self.func.cfg, &self.loopdata.loop_map);
		for block in blocks {
			let block = block.borrow();
			for inst in block.phi_instrs.iter() {
				if !self.tarjan_var.visited.contains(&inst.target) {
					self.run(inst.target.clone());
				}
			}
			for inst in block.instrs.iter() {
				if let Some(t) = inst.get_write() {
					if !self.tarjan_var.visited.contains(&t) {
						self.run(t);
					}
				}
			}
		}
	}
}
