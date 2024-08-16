pub const MAX_INLINE_LENGTH: usize = 4096;
pub const INLINE_PARAMS_THRESHOLD: usize = 50;
pub const GVN_EVAL_NUMBER: usize = 50;
pub const MEM_TO_REG_LIMIT: usize = 1000000;
pub const CONSTANT_SPILL_WEIGHT_RATIO: f64 = 20.0;

pub static VEC_EXTERN: [&str; 17] = [
	"getint",
	"getch",
	"getfloat",
	"getarray",
	"getfarray",
	"putint",
	"putch",
	"putfloat",
	"putarray",
	"putfarray",
	"putf",
	"before_main",
	"after_main",
	"starttime",
	"stoptime",
	"_sysy_starttime",
	"_sysy_stoptime",
];

pub static VEC_MACRO: [&str; 2] = ["starttime", "stoptime"];
pub const MAX_PHI_NUM: usize = 10;

pub const MAX_UNROLL_INSTR_CNT: usize = 200;
pub const MAX_UNROLL_TOTAL_INSTR_CNT: usize = 6000;
pub const CALL_INSTR_CNT: usize = 50;
