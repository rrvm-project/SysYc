use basicblock::basicblock::BasicBlock;
use instruction::InstrSet;

#[allow(unused)]
pub struct CFG {
	basic_blocks: Vec<BasicBlock>,
}
impl CFG {
	pub fn new(basic_blocks: Vec<BasicBlock>) -> CFG {
		CFG { basic_blocks }
	}
	pub fn get_def_and_uses_for(bb: &mut BasicBlock) {
		bb.defs.clear();
		bb.uses.clear();
		if let InstrSet::LlvmInstrSet(instrs) = &mut bb.instrs {
			for i in instrs {
				for itemp in i.get_read() {
					bb.defs.insert(itemp);
				}
				for itemp in i.get_write() {
					bb.defs.insert(itemp);
				}
			}
		}
	}

	pub fn liveliness_analysis(&mut self) {
		for i in self.basic_blocks.iter_mut() {
			Self::get_def_and_uses_for(i);
			i.live_in = i.uses.clone();
			i.live_out.clear();
		}
		let mut is_changed = true;
		while is_changed {
			is_changed = false;
			let mut vec_temp = Vec::new();
			for x in self.basic_blocks.iter() {
				let mut vec_new = Vec::new();
				for j in x.succ.iter() {
					for j_in in self.basic_blocks[*j].live_in.iter() {
						if !x.live_out.contains(j_in) {
							vec_new.push(j_in.clone());
						}
					}
				}
				vec_temp.push(vec_new);
			}
			for (index, x) in self.basic_blocks.iter_mut().enumerate() {
				for itemp in vec_temp[index].iter() {
					x.live_out.insert(itemp.clone());
					if !x.defs.contains(itemp) {
						is_changed = true;
						x.live_in.insert(itemp.clone());
					}
				}
			}
		}
	}
}
