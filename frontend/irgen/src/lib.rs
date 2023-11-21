pub mod irgen;
pub mod utils;

const VALUE: &str = "value";
// 意思是 irgen 过程中会给节点挂上的 attribute，内容是llvm::llvmop::Value, 名字可能名不副实
const IRVALUE: &str = "irvalue";
const FUNC_SYMBOL: &str = "func_symbol";
const SYMBOL: &str = "symbol";
const CUR_SYMBOL: &str = "cur_symbol";
// 数组初始化列表中每一项在数组中的位置
const INDEX: &str = "init_value_index";
const GLOBAL_VALUE: &str = "global_value";
