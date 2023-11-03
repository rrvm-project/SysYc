pub fn trans_indent(input: &str, n: usize) -> String {
	input
		.lines()
		.map(|line| {
			let indent_len = line.chars().take_while(|&c| c == ' ').count();
			let res = &line[indent_len..];
			let new_indent = " ".repeat((indent_len / 4) * n);
			format!("{}{}", new_indent, res)
		})
		.collect::<Vec<_>>()
		.join("\n")
}
