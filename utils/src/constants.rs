pub const MAX_INLINE_LENGTH: usize = 4096;
pub const INLINE_PARAMS_THRESHOLD: usize = 50;

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

pub static EXTEND_TIMES: i32 = 4; // software pipelining 循环展开的次数
pub static DEPENDENCY_EXPLORE_DEPTH: i32 = 10; //  software pipelining 过程中，对于数组的依赖，所枚举到的深度
pub static BLOCKSIZE_THRESHOLD: usize = 100; // software pipelining 判断如果基本本块大小超了 BLOCKSIZE_THRESHOLD 后就不进行针对基本本块的优化
pub static BFS_STATE_THRESHOLD: usize = 7; // 在 instr_scheduling 中，每轮 bfs 所保留的状态的阈值
