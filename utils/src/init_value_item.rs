#[derive(Debug, Clone, Copy)]
pub enum InitValueItem {
	Int(i32),
	Float(f32),
	None(usize),
}

impl InitValueItem {
	pub fn to_i32(&self) -> i32 {
		match self {
			InitValueItem::Int(v) => *v,
			InitValueItem::Float(v) => *v as i32,
			InitValueItem::None(_) => {
				unreachable!("None 类型用于填充初始化列表中空白而不是表示具体的值")
			}
		}
	}

	pub fn to_f32(&self) -> f32 {
		match self {
			InitValueItem::Int(v) => *v as f32,
			InitValueItem::Float(v) => *v,
			InitValueItem::None(_) => {
				unreachable!("None 类型用于填充初始化列表中空白而不是表示具体的值")
			}
		}
	}
}
