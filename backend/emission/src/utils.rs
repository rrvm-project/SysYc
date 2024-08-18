use instruction::RiscvInstrSet;
use utils::{instr_format, mapper::LabelMapper, GlobalVar};

pub const PROGRAM_IDENT: &str = "\"SYSYC: (made by RRVM) 1.0.0\"";

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
	format!(
		"  .align 1
  .type {name}, @function\n{name}:\n{instrs}
  .size {name}, .-{name}"
	)
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

pub fn map_label(mut instrs: RiscvInstrSet, map: &mut LabelMapper) -> String {
	map.map.clear();
	instrs.iter_mut().for_each(|instr| instr.map_label(map));
	instrs.into_iter().map(instr_format).collect::<Vec<_>>().join("\n")
}

pub const RUNTIME_FUNCTION: &str = r#"

.text
.global __create_threads
.global __join_threads

	SYS_clone = 220
	CLONE_VM = 256
	SIGCHLD = 17
	__create_threads:
		li a0, 3   # addi a0, a0, -1
		ble a0, zero, .ret_0
		mv a6, a0
		li a5, 0
		mv a1, sp
		li a2, 0
		li a3, 0
		li a4, 0
	.L0_builtin:
		li a0, (CLONE_VM | SIGCHLD)
		li a7, SYS_clone
		ecall
		blt a0, zero, .try_again
		bne a0, zero, .ret_i
		addi a5, a5, 1
	.try_again:
		blt a5, a6, .L0_builtin
	.ret_n:
		mv a0, a6
		j .L1_builtin
	.ret_0:
		mv a0, zero
		j .L1_builtin
	.ret_i:
		mv a0, a5
	.L1_builtin:
		jr ra

	SYS_waitid = 95
	SYS_exit = 93
	P_ALL = 0
	WEXITED = 4
	__join_threads:
		li a1, 3
		sub a0, a1, a0
		li a1, 4 # new
		mv a4, a0
		addi a5, a1, -1
		beq a4, a5, .L2_builtin
		li a0, P_ALL
		li a1, 0
		li a2, 0
		li a3, WEXITED
		li a7, SYS_waitid
		ecall
	.L2_builtin:
		beq a4, zero, .L3_builtin
		li a0, 0
		li a7, SYS_exit
		ecall
	.L3_builtin:
		jr ra

	

	__fill_zero_words:
		ble a1, zero, .L8_builtin 
		addi a1, a1, -1
		slliw a1, a1, 2
		add a2, a1, a0  # 最后一次4字节
		addi a3, a2, -1
		andi a3, a3, -8 # 最后一次8字节
		andi a4, a0, 7
		beq a4, x0, .L4_builtin

		sw x0, 0(a0)
		addi a0, a0, 4

		.L4_builtin:
			bgtu a0, a3, .L7_builtin 

		.L5_builtin:
			sd x0, 0(a0)
			addi a0, a0, 8
			ble a0, a3, .L5_builtin

		.L7_builtin:
			bgtu a0, a2, .L8_builtin # 如果不够最后一次4字节
			sw x0, 0(a0)
			addi a0, a0, 4

		.L8_builtin:
			jr ra

		

"#;
