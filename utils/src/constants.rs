pub const MAX_INLINE_LENGTH: usize = 4096;
pub const INLINE_PARAMS_THRESHOLD: usize = 50;
pub const GVN_EVAL_NUMBER: usize = 50;

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
