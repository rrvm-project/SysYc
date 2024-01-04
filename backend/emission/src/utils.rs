use utils::GlobalVar;

pub const PROGRAM_IDENT: &str = "\"SYSYC: (made by RRVM) 0.0.1\"";

pub fn program_head(file_name: String) -> String {
	format!(
		"  .file \"{file_name}\"
  .option nopic
  .attribute arch, \"rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0\"
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text"
	)
}

pub fn format_func(name: String, instrs: String) -> String {
	format!("  .align 2
  .global {name}
  .type {name}, @function\n{name}:\n{instrs}
  .size {name}, .-{name}")
}

pub fn format_data(var: GlobalVar) -> String {
	format!(
		"  .global {}\n  .align 2\n  .type {}, @object\n  .size {}, {}\n{}",
		var.ident,
		var.ident,
		var.ident,
		var.size(),
		var
	)
}

pub fn format_bss(var: GlobalVar) -> String {
	format!(
		"  .global {}\n  .align 2\n  .type {}, @object\n  .size {}, {}\n{}:\n  .zero {}",
		var.ident,
		var.ident,
		var.ident,
		var.size(),
		var.ident,
		var.size()
	)
}

pub fn set_section(header: &str, str: String) -> String {
	if let Some(pos) = str.find('\n') {
		let (before, after) = str.split_at(pos + 1);
		format!("{}{}\n{}\n", before, header, after)
	} else {
		"".to_string()
	}
}
