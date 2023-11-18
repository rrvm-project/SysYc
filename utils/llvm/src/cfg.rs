use crate::basicblock::BasicBlock;



#[allow(unused)]
pub struct CFG {
	pub basic_blocks: Vec<BasicBlock>,
    pub entry: usize,
    pub exit: usize,
}

impl CFG {
    pub fn new(entry: BasicBlock) -> CFG {
        CFG { 
            entry: entry.id, 
            basic_blocks: vec![entry], 
            exit: 0 }
    }

    pub fn set_exit(&mut self, exit: BasicBlock) {
        self.exit = exit.id;
        self.basic_blocks.push(exit);
    }
    
}