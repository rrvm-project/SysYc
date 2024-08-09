use super::IndVarSolver;

impl<'a> IndVarSolver<'a> {
	pub fn move_invariant(&mut self) {
		for block in
			self.cur_loop.borrow().blocks_without_subloops(self.cfg, self.loop_map)
		{
			for inst in block.borrow().instrs.iter() {
				if let Some(t) = inst.get_write() {
					if self.loop_invariant.contains(&t) {
						eprintln!("moving invariant: {}", t);
						self.preheader.borrow_mut().instrs.push(inst.clone());
						*self.def_map.get_mut(&t).unwrap() = self.preheader.clone();
						self.flag = true;
					}
				}
			}
			block.borrow_mut().instrs.retain(|inst| {
				!inst.get_write().is_some_and(|t| self.loop_invariant.contains(&t))
			});
		}
	}
}
