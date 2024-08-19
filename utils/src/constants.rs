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

pub static EXTEND_TIMES: i32 = 4; // software pipelining 循环展开的次数
pub static DEPENDENCY_EXPLORE_DEPTH: i32 = 10; //  software pipelining 过程中，对于数组的依赖，所枚举到的深度
pub static BLOCKSIZE_THRESHOLD: usize = 100; // software pipelining 判断如果基本本块大小超了 BLOCKSIZE_THRESHOLD 后就不进行针对基本本块的优化
pub static BFS_STATE_THRESHOLD: usize = 3; // 在 instr_scheduling 中，每轮 bfs 所保留的状态的阈值
																					 // for instruction scheduling: register punishment
pub static ADD_ALLOCATABLES: i32 = 0;
pub static NEAR_END: i32 = 0; // 寄存器生命周期更快结束的指令优先
pub static REDUCE_SUB: i32 = 0; // 后继中的节点对应指令，寄存器生命周期更快结束的指令优先
pub static REDUCE_LIVE: i32 = 0;
pub static LIVE_THROUGH: usize = 30;
pub static SUM_MIN_RATIO: i32 = 1;
pub static SCHEDULE_THRESHOLD: usize = 15000;
pub static SOFTWARE_PIPELINE_PARAM: i32 = 0; // 拓扑排序后软流水的权重
pub static HARDWARE_PIPELINE_PARAM: i32 = 1; // 拓扑排序后硬件流水的权重
pub static FDIV_WAIT: usize = 20; // fdiv 的 repeat rate
