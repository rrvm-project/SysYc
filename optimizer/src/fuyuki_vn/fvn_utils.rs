#[derive(Debug)]
pub struct MaxMin<T> {
	max: Option<T>,
	min: Option<T>,
}

impl<T: PartialOrd + Clone + Copy> MaxMin<T> {
	pub fn new_with_init(n: T) -> Self {
		MaxMin {
			max: Some(n),
			min: Some(n),
		}
	}
	pub fn update(&mut self, n: T) -> &Self {
		if let Some(max) = self.max {
			if max < n {
				self.max = Some(n);
			}
		} else {
			self.max = Some(n);
		}
		if let Some(min) = self.min {
			if min > n {
				self.min = Some(n);
			}
		} else {
			self.min = Some(n);
		}
		self
	}

	pub fn max(&self) -> T {
		self.max.unwrap()
	}

	pub fn min(&self) -> T {
		self.min.unwrap()
	}
}
