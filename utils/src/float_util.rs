struct Float32 {
	pub sign: i8,
	pub exponent: i16, //8bits
	pub mantissa: u32, // 23bits
}

impl std::fmt::Debug for Float32 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{} {} {:023b}",
			self.sign, self.exponent, self.mantissa
		))
	}
}

impl From<f32> for Float32 {
	fn from(input: f32) -> Self {
		assert!(!input.is_nan());
		let bits: u32 = input.to_bits();
		let sign: i8 = if bits >> 31 == 0 { 1 } else { -1 };
		let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
		let mantissa = bits & 0x7fffff;

		// Exponent bias + mantissa shift
		exponent -= 127;

		Float32 {
			sign,
			exponent,
			mantissa,
		}
	}
}

impl Float32 {
	fn is_zero(&self) -> bool {
		self.exponent == -127 && self.mantissa == 0
	}

	fn is_inf(&self) -> bool {
		self.exponent == 128
	}

	fn increase_abs(&mut self) {
		if self.is_inf() {
			return;
		}
		self.mantissa += 1;
		if self.mantissa == 0x800000 {
			self.mantissa = 0;
			self.exponent += 1;
		}
		if self.is_inf() {
			self.mantissa = 0;
		}
	}

	fn decrease_abs(&mut self) {
		assert!(!self.is_zero());
		if self.is_inf() {
			self.exponent = 127;
			self.mantissa = 0x7fffff;
			return;
		}

		if self.mantissa == 0 {
			self.mantissa = 0x800000;
			self.exponent -= 1;
		}
		self.mantissa -= 1;
	}

	pub fn increase(&mut self) {
		if self.is_zero() {
			self.exponent = -127;
			self.mantissa = 1;
			self.sign = 1;
		} else if self.sign == 1 {
			self.increase_abs();
		} else {
			self.decrease_abs();
		}
	}

	pub fn decrease(&mut self) {
		if self.is_zero() {
			self.exponent = -127;
			self.mantissa = 1;
			self.sign = -1;
		} else if self.sign == 1 {
			self.decrease_abs();
		} else {
			self.increase_abs();
		}
	}

	pub fn to_bits(&self) -> u32 {
		let mut result: u32 = self.mantissa;
		result &= 0x7fffff;
		result |= ((self.exponent + 127) as u32) << 23;
		if self.sign == -1 && !self.is_zero() {
			result |= 0x8000_0000;
		}
		result
	}
}

pub fn f32_add_eps(v: f32) -> f32 {
	if v.is_nan() {
		return v;
	}
	let mut a: Float32 = v.into();
	a.increase();
	f32::from_bits(a.to_bits())
}

pub fn f32_sub_eps(v: f32) -> f32 {
	if v.is_nan() {
		return v;
	}
	let mut a: Float32 = v.into();
	a.decrease();
	f32::from_bits(a.to_bits())
}
