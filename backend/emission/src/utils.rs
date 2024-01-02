use utils::GlobalVar;

pub const PROGRAM_IDENT: &str = "\"SYSYC: (made by RRVM) 0.0.1\"";

pub fn program_head(file_name: String) -> String {
	format!(
		"  .file \"{file_name}\"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text"
	)
}

pub fn format_func(name: String, instrs: String) -> String {
	format!("  .align 1\n  .global {name}\n  .type {name}, @function\n{name}:\n{instrs}")
}

pub fn format_data(var: GlobalVar) -> String {
	format!(
		"  .global {}\n  .type {}, @object\n  .size {}, {}\n{}",
		var.ident,
		var.ident,
		var.ident,
		var.size(),
		var
	)
}

pub fn format_bss(var: GlobalVar) -> String {
	format!(
		"  .global {}\n  .type {}, @object\n  .size {}, {}\n{}:\n  .space {}",
		var.ident,
		var.ident,
		var.ident,
		var.size(),
		var.ident,
		var.size()
	)
}

pub fn set_section(header: &str, str: String) -> String {
	if str.is_empty() {
		"".to_string()
	} else {
		format!("{header}\n{str}\n")
	}
}