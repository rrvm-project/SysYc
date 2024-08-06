use llvm::{LlvmTemp, Value, VarType};
use std:: fmt::Write;

const MAX_SIZE: usize = 64;

#[derive(PartialEq, Eq, PartialOrd, Clone)]
enum Single {
	Int(i32),
	Temp(LlvmTemp),
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum GraphOp {
	Plus,
	Mul,
}

impl GraphOp{
	pub fn eval (&self, x1: i32, x2: i32) -> i32 {
		match  self {
			GraphOp::Mul => x1 * x2,
			GraphOp::Plus => x1 + x2
		}
	}
}

impl std::fmt::Debug for GraphOp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Plus => f.write_str("+"),
			Self::Mul => f.write_str("*"),
		}
	}
}

impl std::fmt::Debug for Single {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Int(arg0) => f.write_str(format!(" {} ", arg0).as_str()),
			Self::Temp(arg0) => f.write_str(format!(" {:?} ", arg0).as_str()),
		}
	}
}

impl Ord for Single {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(Single::Int(_), Single::Temp(_)) => std::cmp::Ordering::Less,
			(Single::Temp(_), Single::Int(_)) => std::cmp::Ordering::Greater,
			(Single::Int(a), Single::Int(b)) => a.cmp(b),
			(Single::Temp(a), Single::Temp(b)) => a.name.cmp(&b.name),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Clone)]
enum GraphValue {
	Single(Single),
	NonTrival((GraphOp, Vec<GraphValue>)),
}

impl GraphValue {
	fn as_number(&self) -> Option<i32> {
		match self {
			GraphValue::Single(Single::Int(i)) => Some(*i),
			_ => None
		}
	}
	fn as_single(&self) -> Option<&Single> {
		match self {
			GraphValue::Single(s) => Some(s),
			_ => None
		}
	}

	fn as_non_trival(&self, op_target: GraphOp) -> Option<&Vec<GraphValue>>{
		match self {
			GraphValue::NonTrival((op, v)) if *op == op_target => {
				Some(v)
			},
			_ => None 
		}
	}

	fn as_non_trival_mut(&mut self, op_target: GraphOp) -> Option<&mut Vec<GraphValue>>{
		match self {
			GraphValue::NonTrival((op, v)) if *op == op_target => {
				Some(v)
			},
			_ => None 
		}
	}

}

struct GraphValueCollectIterator<'a> {
	index: usize,
	op: GraphOp,
	my_struct: &'a GraphValue,
}

impl<'a> GraphValue {
	fn collect(&'a self, op: GraphOp) -> GraphValueCollectIterator<'a> {
		GraphValueCollectIterator {
			index: 0,
			my_struct: self,
			op,
		}
	}
}

impl<'a> Iterator for GraphValueCollectIterator<'a> {
	type Item = &'a GraphValue; fn next(&mut self) -> Option<Self::Item> { let result = self.peek();
		self.index += 1;
		result
	}
}

impl<'a> GraphValueCollectIterator<'a> {
	fn peek(&mut self) -> Option<&'a GraphValue> {
		match self.my_struct {
			GraphValue::NonTrival((op, v)) if *op == self.op || v.len() < 2=> v.get(self.index),
			_ => {
				if self.index == 0 {
					Some(self.my_struct)
				} else {
					None
				}
			}
		}
	}
}

impl GraphValue {
	pub fn size(&self) -> usize {
		match self {
			GraphValue::Single(_) => 1,
			GraphValue::NonTrival((_, graph)) => {
				graph.iter().map(GraphValue::size).sum()
			}
		}
	}
}

impl std::fmt::Debug for GraphValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Single(s) => f.write_str(format!("{:?}", s).as_str()),
			Self::NonTrival((op, v)) => {
                let left_brace = if *op == GraphOp::Mul {'['} else {'('};
                let right_brace = if *op == GraphOp::Mul {']'} else {')'};
				f.write_char(left_brace).unwrap();
				v.iter().enumerate().for_each(|(i, v)| {
					if i > 0 {
						f.write_str(format!("{:?}", op).as_str()).unwrap();
					}
					f.write_str(format!("{:?}", v).as_str()).unwrap();
				});
				f.write_char(right_brace)
			}
		}
	}
}

impl Ord for GraphValue {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(GraphValue::Single(_), GraphValue::NonTrival(_)) => {
				std::cmp::Ordering::Less
			}
			(GraphValue::NonTrival(_), GraphValue::Single(_)) => {
				std::cmp::Ordering::Greater
			}
			(GraphValue::Single(a), GraphValue::Single(b)) => a.cmp(b),
			(GraphValue::NonTrival((opa, va)), GraphValue::NonTrival((opb, vb))) => {
				match opa.cmp(opb) {
					std::cmp::Ordering::Equal => va
						.iter()
						.zip(vb.iter())
						.try_fold(va.len().cmp(&vb.len()), |last, (a, b)| match last {
							std::cmp::Ordering::Equal => Ok(a.cmp(b)),
							_ => Err(last), //Err相当于Break!
						})
						.err()
						.unwrap_or(std::cmp::Ordering::Equal),
					other => other,
				}
			}
		}
	}
}

fn gcd(this: &GraphValue, other: &GraphValue) -> Option<GraphValue> {
	// assert the values are sorted
	let mut left = this.collect(GraphOp::Mul);
	let mut right = other.collect(GraphOp::Mul);

	let mut result = vec![];

	while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
		if a == b {
			if a.as_number().is_none() {
				result.push(a.clone());
			}
			left.next();
			right.next();
		} else if a < b {
			left.next();
		} else if a > b {
			right.next();
		} else {
			unreachable!();
		}
	}

	if result.len() == 0 {
		None
	} else {
		Some(GraphValue::NonTrival((GraphOp::Mul, result)))
	}
}

pub fn div(this: &GraphValue, other: &GraphValue) -> Option<GraphValue> {
	let mut left = this.collect(GraphOp::Mul);
	let mut right = other.collect(GraphOp::Mul);

	let mut result = vec![];

	while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
		if a == b {
			left.next();
			right.next();
		} else if a < b {
			result.push(left.next().unwrap().clone());
		} else if a > b {
			return None;
		} else {
			unreachable!();
		}
	}

	while let Some(v) = left.next() {
		result.push(v.clone());
	}

	if result.is_empty() {
		GraphValue::Single(Single::Int(1)).into()
	} else {
		GraphValue::NonTrival((GraphOp::Mul, result)).into()
	}
}

fn solve_constant(v : &mut GraphValue) -> bool {
	let old_size = v.size();
	
	match v {
		GraphValue::Single(_) => {return false},
		GraphValue::NonTrival((op, v)) => {

			let start = match op {
				GraphOp::Mul => 1i32,
				GraphOp::Plus => 0i32
			};

			let mut value = start;

			let mut new_vec = vec![];

			while let Some(v) = v.pop(){
				if let Some(i) = v.as_number(){
					value = op.eval(value, i);
				} else {
					new_vec.push(v);
				}
			}

			if value == 0i32 && *op == GraphOp::Mul{
				new_vec.clear();
			}

			if value != start || *op == GraphOp::Plus{
				v.push(GraphValue::Single(Single::Int(value)));
			}

			while let Some(item) = new_vec.pop(){
				v.push(item);	
			}

		}
	}

	v.size() < old_size
}

// return the (index, size) of biggest muli part in target. 
fn can_add_para(v: &Vec<Option<GraphValue>>, target: &GraphValue) -> Option<(usize, usize)>{
	if target.as_single().is_some(){
		return None;
	}

	if let Some(vec) = target.as_non_trival(GraphOp::Mul){
		let mut max = None;

		for (i, muli_part) in vec.iter().enumerate(){
			if let Some(factors) = muli_part.as_non_trival(GraphOp::Plus){
				let mut total = v.iter().filter_map(|i: &Option<GraphValue>| i.as_ref()).filter(|i| i.as_number().is_none()).peekable();
				let mut to_find = factors.iter().filter(|i| i.as_number().is_none()).peekable();
				while let (Some(a), Some(b)) = (total.peek(), to_find.peek()){
					if a == b{
						total.next();
						to_find.next();
					} else if a < b{
						total.next();
					} else {
						break;
					}
				}

				if to_find.peek().is_none(){
					let this_size = muli_part.size();
					if let Some((_, max_size)) = max {
						if max_size < this_size {
							max = Some((i, this_size))
						}
					} else {
						max = Some((i, this_size))
					}
				}

			}
		}

		return max;
	} else {
		return None;
	}		
}


fn add_para(v: &mut Vec<GraphValue>, target: &mut Vec<GraphValue>){
	v.sort();
	target.sort();
	let new_v = std::mem::take(v);

	let mut left = new_v.into_iter().peekable();
	let mut right = target.iter().peekable();

	let mut const_part_remain = 0i32;
	let mut const_part_in_para = 0i32;
	let mut in_para = vec![];

	while let (Some(l), Some(r)) = (left.peek(), right.peek()) {
		if let Some(li) = l.as_number(){
			const_part_remain += li;
			left.next();
			continue;
		}
		if let Some(ri) = r.as_number() {
			const_part_remain -= ri;
			const_part_in_para += ri;
			right.next();
			continue;
		}

		if *l == **r{
			in_para.push(left.next().unwrap());
			right.next();
		} else if *l < **r {
			v.push(left.next().unwrap());
		} else {
			unreachable!()
		}

	}

	assert!(right.peek().is_none());

	while let Some(l) = left.next(){
		v.push(l);
	}

	if const_part_remain != 0{
		v.push(GraphValue::Single(Single::Int(const_part_remain)));
		// pushing the para to v will definately make it not sorted, so never mind that v is not sorted here.
	}

	let mut para = vec![];

	if const_part_in_para != 0{
		para.push(GraphValue::Single(Single::Int(const_part_in_para)));
	}

	para.append(&mut in_para);

	v.push(GraphValue::NonTrival((GraphOp::Plus, para)));
	v.sort();

}


fn make_para(v: &mut Vec<GraphValue>) {
	let mut new_vec = vec![];
	for item in std::mem::take(v) {
		new_vec.push(Some(item));
	}

	let mut max : Option<(usize, usize, usize)> = None;
	for i in 0..new_vec.len(){
		let value_i = new_vec[i].take().unwrap();
		
		if let Some((inner_index, size)) = can_add_para(&new_vec, &value_i){
			if !max.is_some_and(|(_, _, old_max)| old_max > size) {
				max = Some((i, inner_index, size));
			}
		}

		new_vec[i] = Some(value_i);		
	}

	if let Some((value_index, inner_index, _)) = max {
		let mut value_i = new_vec[value_index].take().unwrap();

		if let Some(target) = value_i.as_non_trival_mut(GraphOp::Mul){
			let mut remain_v : Vec<GraphValue> = new_vec.into_iter().filter_map(|v|v).collect();
			// dbg!(&remain_v);
			let inner = target.get_mut(inner_index).unwrap();
			if let Some(inner) = inner.as_non_trival_mut(GraphOp::Plus){
				add_para(&mut remain_v, inner);
			}

			remain_v.push(value_i);

			
			remain_v.sort();
			// dbg!(&remain_v);
			*v = remain_v;
			return;
		}
		unreachable!();
	} else  {
		new_vec.into_iter().for_each(|item| v.push(item.unwrap()));
	}
	
} 


fn distributive_law(v: &mut Vec<GraphValue>) -> bool {
	let mut result = vec![];
	let mut changed = false;
	while let Some(new) = v.pop() {
		let mut pending: Option<(GraphValue, Vec<GraphValue>)> = None;
		for item in std::mem::take(&mut result) {
			pending = match (gcd(&new, &item), pending) {
				(None, pending) => {
					result.push(item);
					pending
				}
				(Some(g), None) => Some((g, vec![item])),
				(Some(new_g), Some((old_g, mut old_vec))) => {
					dbg!(&new_g, &old_g, &old_vec);
					if new_g < old_g {
						result.push(item);
						Some((old_g, old_vec))
					} else if new_g == old_g {
						old_vec.push(item);
						Some((old_g, old_vec))
					} else {
						result.append(&mut old_vec);
						Some((new_g, vec![item]))
					}
				}
			};
		}

		if let Some((mut g, mut items)) = pending {
			g.sort();
			let mut remains = vec![];
			items.push(new);

			for item in items {
				let remain = div(&item, &g).unwrap();
				remains.push(remain);
			}

			let mut product_vec: Vec<GraphValue> =
				g.collect(GraphOp::Mul).cloned().collect();
			product_vec.push(GraphValue::NonTrival((GraphOp::Plus, remains)));

			changed = true;
			product_vec.sort();
			result.push(GraphValue::NonTrival((GraphOp::Mul, product_vec)));
		} else {
			result.push(new);
		}
	}

	result.sort();
	*v = result;
	changed
}

impl GraphValue {
	fn sort(&mut self) {
		match self {
			GraphValue::Single(_) => {}
			GraphValue::NonTrival((op, v)) => {
				v.iter_mut().for_each(GraphValue::sort);
				v.sort();
			}
		}
	}

	fn collect_with_op(self, output: &mut Vec<GraphValue>, op: GraphOp) {
		match self {
			GraphValue::NonTrival((op_this, v)) => {
				if op_this == op || v.len() < 2{
					for item in v {
						item.collect_with_op(output, op);
					}
				} else {
					output.push(GraphValue::NonTrival((op_this, v)));
				}
			}
			_ => {
				output.push(self);
			}
		}
	}

	fn reduce(&mut self) {
        
        let mut reduce_self = None;
		match self {
			GraphValue::Single(_) => {}
			GraphValue::NonTrival((op, v)) => {
				v.iter_mut().for_each(GraphValue::reduce);

				let mut new_v = vec![];
				std::mem::take(v).into_iter().for_each({
					|value| {
						value.collect_with_op(&mut new_v, *op);
					}
				});
				
                if new_v.len() == 1{
                    reduce_self = new_v.pop();
                } else {
                    *v = new_v;
                }
			}
		}

        if let Some(reduce_self) = reduce_self{
            *self = reduce_self;
        }
        
	}

    fn simplify(&mut self) -> bool{
        let mut changed = false;

        match self{
            GraphValue::Single(_) => {},
            GraphValue::NonTrival((op, v)) => {
                for item in v.iter_mut(){
                    changed |= item.simplify();
                }
                if *op == GraphOp::Plus {
					make_para(v);
                    changed |= distributive_law(v);
                }
            },
        }
		changed |= solve_constant(self);
        changed
    }

	fn sanity(&mut self){
		loop {
			self.sort();
			self.reduce();
			dbg!(&self);
			if !self.simplify() {
				break;
			}
			dbg!(&self);
		}
	}

	pub fn check_over_size(self) -> Option<Self>{
		if self.size() <= MAX_SIZE{
			Some(self)
		} else {
			None
		}
	}


	pub fn add(&self, other: &GraphValue) -> Option<GraphValue>{
		let mut result = GraphValue::NonTrival((GraphOp::Plus, vec![
			self.clone(),
			other.clone(),
		]));



		result.sanity();

		result.check_over_size()
	}

	pub fn sub(&self, other: &GraphValue) -> Option<GraphValue>{
		let mut result = GraphValue::NonTrival((GraphOp::Plus, vec![
			self.clone(),
			GraphValue::NonTrival((GraphOp::Mul, vec![
				GraphValue::Single(Single::Int(-1)),
				other.clone()
			]))
		]));

		result.sanity();
		result.check_over_size()
	}

	pub fn mul(&self, other: &GraphValue) -> Option<GraphValue>{
		let mut result = GraphValue::NonTrival((GraphOp::Mul, vec![
			self.clone(),
			other.clone(),
		]));

		result.sanity();

		result.check_over_size()
	}


	pub fn div(&self, other: &GraphValue) -> Option<GraphValue>{
		if let Some(mut result) = div(self, other) {
			result.sanity();
			Some(result)
		}else{
			None
		}
	}


	fn from_value(value: Value)	-> Option<GraphValue>{
		match value{
			Value::Int(i) => Some(GraphValue::Single(Single::Int(i))),
			Value::Temp(t) if t.var_type == VarType::I32 => Some(GraphValue::Single(Single::Temp(t))),
			_ => None
		}
	}
}



#[cfg(test)]
mod tests {
	use std::vec;

	use llvm::VarType;

	use super::*; // 导入主模块中的所有内容

	fn get_tmp(id: usize) -> Single {
		Single::Temp(LlvmTemp {
			name: format!("{}", id),
			is_global: false,
			var_type: VarType::I32,
		})
	}

	#[test]
	fn test_distributive() {
		let mut b = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(5)),
				GraphValue::Single(get_tmp(5)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![
						GraphValue::Single(Single::Int(6)),
						GraphValue::Single(get_tmp(6)),
					],
				)),
			],
		));

		let mut c = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(1000)),
				GraphValue::Single(get_tmp(5)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![
						GraphValue::Single(Single::Int(6)),
						GraphValue::Single(get_tmp(6)),
					],
				)),
			],
		));

		let mut a = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![b.clone(), GraphValue::Single(Single::Int(3))],
				)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![b.clone(), GraphValue::Single(Single::Int(3))],
				)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![b.clone(), GraphValue::Single(get_tmp(33))],
				)),
                b.clone(),
				GraphValue::Single(get_tmp(23423)),
				c.clone()
			],
		));

        dbg!(&a);

        // a.sort();
        // a.reduce();
        // a.simplify();
        // a.sort();
        // a.reduce();

        // dbg!(&a);

		a.sanity();


		dbg!(&a);
        
	}

	#[test]
	fn test_reduce() {
		let mut a = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(-100)),
				GraphValue::Single(get_tmp(3)),
				GraphValue::NonTrival((
					GraphOp::Plus,
					vec![
						GraphValue::Single(Single::Int(4)),
						GraphValue::Single(get_tmp(4)),
						GraphValue::NonTrival((
							GraphOp::Mul,
							vec![
								GraphValue::Single(Single::Int(5)),
								GraphValue::Single(get_tmp(5)),
								GraphValue::NonTrival((
									GraphOp::Plus,
									vec![
										GraphValue::Single(Single::Int(6)),
										GraphValue::Single(get_tmp(6)),
									],
								)),
							],
						)),
						GraphValue::NonTrival((
							GraphOp::Mul,
							vec![
								GraphValue::Single(Single::Int(14)),
								GraphValue::Single(get_tmp(14)),
								GraphValue::NonTrival((
									GraphOp::Mul,
									vec![
										GraphValue::Single(Single::Int(15)),
										GraphValue::Single(get_tmp(15)),
										GraphValue::NonTrival((
											GraphOp::Mul,
											vec![
												GraphValue::Single(Single::Int(16)),
												GraphValue::Single(get_tmp(16)),
											],
										)),
									],
								)),
							],
						)),
						GraphValue::NonTrival((
							GraphOp::Plus,
							vec![
								GraphValue::Single(Single::Int(14)),
								GraphValue::Single(get_tmp(14)),
								GraphValue::NonTrival((
									GraphOp::Plus,
									vec![
										GraphValue::Single(Single::Int(15)),
										GraphValue::Single(get_tmp(15)),
										GraphValue::NonTrival((
											GraphOp::Plus,
											vec![
												GraphValue::Single(Single::Int(16)),
												GraphValue::Single(get_tmp(16)),
											],
										)),
									],
								)),
							],
						)),
					],
				)),
			],
		));

		dbg!(&a);

		a.reduce();

		dbg!(&a);
	}

	#[test]
	fn test_mul_1(){
		let mut b = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(5)),
				GraphValue::Single(get_tmp(5)),
				GraphValue::NonTrival((
					GraphOp::Mul,
					vec![
						GraphValue::Single(Single::Int(6)),
						GraphValue::Single(get_tmp(6)),
					],
				)),
			],
		));

		let mut c = GraphValue::NonTrival((
			GraphOp::Plus,
			vec![
				GraphValue::Single(Single::Int(82)),
				GraphValue::Single(get_tmp(5)),
			],
		));

		let mut a = GraphValue::NonTrival((GraphOp::Mul, vec![b.clone(), c]));

		a.sanity();


		dbg!(&a);


		let c = div(&a,&b);

		dbg!(&c);

	}

	#[test]
	fn test_sub(){
		let a = GraphValue::Single(get_tmp(2));
		let b = GraphValue::Single(get_tmp(3));


		let k = a.sub(&b).unwrap();
		let g = b.sub(&a).unwrap();

		let h = k.add(&g);

		dbg!(k, g, h);
		// cargo test --package optimizer --lib -- arith::compute_graph::tests::test_sub --exact --show-output 

	}
}
