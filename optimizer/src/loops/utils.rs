use rrvm::rrvm_loop::LoopPtr;

pub fn print_all_loops(root: LoopPtr) {
	let mut queue = vec![root];
	while let Some(loop_) = queue.pop() {
		println!("loop: {}", loop_.borrow().header.borrow().label());
		for subloop in loop_.borrow().subloops.iter() {
			queue.push(subloop.clone());
		}
	}
}
