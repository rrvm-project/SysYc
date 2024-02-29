use crate::simulator::StackValue;

fn putint(output: &mut Vec<String>, data: i32) -> (bool, Option<StackValue>) {
	output.push(format!("{:?}", data));
	(true, None)
}

fn is_blank(char: i32) -> bool {
	!(char > ' ' as i32 && char < 127)
}

fn jump_blank(get_chr: &mut impl FnMut(bool) -> Option<i32>) {
	loop {
		let char = get_chr(true);
		if char.is_none() {
			break;
		}
		if is_blank(char.unwrap()) {
			get_chr(false);
		} else {
			break;
		}
	}
}

fn getint(get_chr: &mut impl FnMut(bool) -> Option<i32>) -> Option<i32> {
	let mut is_negative = false;
	let mut result: i32 = 0;
	let mut ok_flag = false;

	jump_blank(get_chr);

	let negativemark = '-' as i32;
	let c = get_chr(true);
	if c.unwrap_or_default() == negativemark {
		is_negative = true;
		get_chr(false);
	}

	loop {
		let c = get_chr(true);
		c?;
		let c = c.unwrap();
		match c {
			48..=57 => {
				result *= 10;
				result += c - 48;
				ok_flag = true;
			}
			_ => {
				break;
			}
		}
		get_chr(false);
	}

	if ok_flag {
		if is_negative {
			Some(-result)
		} else {
			Some(result)
		}
	} else {
		None
	}
}

pub fn inout(
	name: &str,
	params: &[StackValue],
	output: &mut Vec<String>,
	input: &str,
	input_position: &mut usize,
) -> (bool, Option<StackValue>) {
	let mut get_chr = |peek: bool| {
		if *input_position < input.len() {
			let result = Some(input.chars().nth(*input_position).unwrap() as i32);
			if !peek {
				*input_position += 1;
			}
			result
		} else {
			None
		}
	};

	match name {
		"putint" => putint(output, params[0].as_i32()),
		"getch" => (true, get_chr(false).map(StackValue::from)),
		"getint" => (true, getint(&mut get_chr).map(StackValue::from)),
		_ => (false, None),
	}
}
