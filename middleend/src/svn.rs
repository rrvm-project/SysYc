use crate::{context::IRPassContext, irpass::IRPass};
use llvm::{cfg::CFG, LlvmProgram, llvmop::LlvmOp};
use std::collections::HashSet;
pub struct Svn {}

impl IRPass for Svn {
	fn pass(&mut self, program: &mut LlvmProgram, context: &mut IRPassContext) {
		for item in &mut program.funcs {
			let cfg = &mut item.cfg;
            println!("\n\nfor function {}", item.label);
			&mut self.traverse_cfg(cfg, context);
		}
	}
}

struct LvnDataItem {
    pub operator : 
}
struct BasicBlockLvnData {
    pub id: usize,
    pub data : HashMap<>
}

impl Svn {
	fn traverse_cfg(&mut self, cfg: &mut CFG, ctx: &mut IRPassContext) {
		let mut work_list = vec![cfg.entry];
        let mut visited = HashSet::<usize>::new();

		let mut id;
        let mut lvn_data : Vec<BasicBlockLvnData> = vec![];

		while !work_list.is_empty() {
			id = work_list.pop().unwrap();
			self.svn(id, cfg, &mut work_list, &mut lvn_data, &mut visited,ctx);
		}

        // println!("cfg");
        // for item in &cfg.basic_blocks{
        //     println!("item id {}", item.id);
        // }
	}

	fn svn(
		&mut self,
		id: usize,
		cfg: &mut CFG,
		work_list: &mut Vec<usize>,
        lvn_data : &mut Vec<BasicBlockLvnData>,
        visited: &mut HashSet<usize>,
		ctx: &mut IRPassContext,
	) {
        lvn_data.push(BasicBlockLvnData { id });

        visited.insert(id);
        self.lvn(id, cfg, lvn_data, ctx);

        let mut to_visit_next = vec![];
        for succ in &cfg.basic_blocks[id].succ{
            if cfg.basic_blocks[*succ].pred.len() == 1 {
                to_visit_next.push(*succ);
            } else if !visited.contains(succ) {
                work_list.push(*succ);
                println!("pushed into worklist {}", *succ);
            }
        }

        for item in to_visit_next {
            self.svn(item, cfg, work_list, lvn_data, visited, ctx);
        }

        lvn_data.pop();
	}

    fn lvn(
        &mut self,
        id: usize,
        cfg: &mut CFG,
        lvn_data : &mut Vec<BasicBlockLvnData>,
        _: &mut IRPassContext
    ) {
        print!("visited basic block {} \n [", id);
        for item in lvn_data{
            print!("{} ", item.id);
        }
        println!("]")
    }
}
