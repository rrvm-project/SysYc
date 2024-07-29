use std::collections::{HashMap, HashSet, VecDeque};

use super::PureCheck;
use crate::RrvmOptimizer;
use rrvm::{func::Entrance, program::LlvmProgram};
use utils::errors::Result;

impl RrvmOptimizer for PureCheck {
	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		let mut work_list: VecDeque<String> = VecDeque::new();
		let mut inverse_call_relation: HashMap<String, HashSet<String>> =
			HashMap::new();
		let mut call_relation: HashMap<String, HashSet<String>> = HashMap::new();

		// program.funcs.iter_mut().map(|func| mark(func));
		for func in program.funcs.iter_mut() {
			for block in &func.cfg.blocks {
				for instr in &block.borrow().instrs {
					if let Some(item) = instr.external_resorce() {
						// func.external_resorce.insert(item);
						match item {
							utils::purity::ExternalResource::Call(callee) => {
								inverse_call_relation
									.entry(callee.clone())
									.or_default()
									.insert(func.name.clone());
								call_relation
									.entry(func.name.clone())
									.or_default()
									.insert(callee.clone());
							}
							v => {
								work_list.push_front(func.name.clone());
								func.external_resorce.insert(v);
							}
						}
					}
				}
			}
		}

		let mut begin: HashMap<String, usize> = HashMap::new();
		let mut end: HashMap<String, usize> = HashMap::new();

		let mut timer: usize = 0;
		let mut in_loop: HashSet<String> = HashSet::new();

		fn dfs(
			name: &String,
			timer: &mut usize,
			begin: &mut HashMap<String, usize>,
			end: &mut HashMap<String, usize>,
			call_relation: &HashMap<String, HashSet<String>>,
			result: &mut HashSet<String>,
		) -> Option<usize> {
			*timer += 1;
			begin.insert(name.clone(), *timer);
			let self_begin = *timer;

			let mut earliest: Option<usize> = None;

			let update = |old: Option<usize>, new: Option<usize>| match (old, new) {
				(None, _) => new,
				(Some(_), None) => old,
				(Some(u), Some(v)) => Some(u.min(v)),
			};

			for callee in
				call_relation.get(name).iter().flat_map(|callees| callees.iter())
			{
				match (begin.get(callee), end.get(callee)) {
					(None, None) => {
						//tree edge
						update(
							earliest,
							dfs(callee, timer, begin, end, call_relation, result),
						);
					}
					(None, _) => unreachable!(),
					(Some(u), None) => {
						//back edge, (also self loop)
						earliest = update(earliest, Some(*u));
					}
					(Some(_), Some(_)) => {
						// forward or side
					}
				}
			}

			if let Some(u) = earliest {
				if u <= self_begin {
					result.insert(name.clone());
				}
			}

			*timer += 1;
			end.insert(name.clone(), *timer);
			earliest
		}

		dfs(
			&"main".to_string(),
			&mut timer,
			&mut begin,
			&mut end,
			&call_relation,
			&mut in_loop,
		);

		for func in program.funcs.iter_mut() {
			let name = &func.name;
			func.entrance = match (begin.contains_key(name), in_loop.contains(name)) {
				(true, true) => Entrance::Multi,
				(true, false) => Entrance::Single,
				(false, _) => Entrance::Never,
			};
		}

		// for func in program.funcs.iter_mut(){
		// 	let name = &func.name;
		// 	dbg!((name, func.entrance));
		// }

		let mut not_pure = HashSet::new();
		while let Some(funcname) = work_list.pop_back() {
			if not_pure.contains(&funcname) {
				continue;
			}
			not_pure.insert(funcname.clone());
			if let Some(item) = inverse_call_relation.get(&funcname) {
				for func in item {
					if not_pure.contains(func) {
						continue;
					}
					work_list.push_back(func.clone());
				}
			}
		}

		for func in program.funcs.iter_mut() {
			for block in func.cfg.blocks.iter() {
				for instr in block.borrow().instrs.iter() {
					if let Some(item) = instr.external_resorce() {
						// func.external_resorce.insert(item);
						// match &item {
						// 	utils::purity::ExternalResource::Call(callee) => {
						// 		if not_pure.contains(callee) {
						// 			func.external_resorce.insert(item);
						// 		}
						// 	}
						// 	_ => {}
						// }
						if let utils::purity::ExternalResource::Call(callee) = &item {
							if not_pure.contains(callee) {
								func.external_resorce.insert(item);
							}
						}
					}
				}
			}
		}

		program.funcs.retain(|func| func.entrance != Entrance::Never);

		Ok(false)
	}
}
