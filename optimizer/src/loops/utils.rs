use rrvm::rrvm_loop::LoopPtr;

#[allow(unused)]
pub fn print_all_loops(root: LoopPtr) {
	let mut queue = vec![root];
	while let Some(loop_) = queue.pop() {
		println!("loop: {}, outer: {:?}", loop_.borrow().header.borrow().label(), loop_.borrow().outer.clone().map(|l| l.upgrade().unwrap().borrow().header.borrow().label()));
		for subloop in loop_.borrow().subloops.iter() {
			queue.push(subloop.clone());
		}
	}
}
