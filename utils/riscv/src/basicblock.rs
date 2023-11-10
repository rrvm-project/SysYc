use llvm::label::Label;
use std::collections::BTreeSet;
use llvm::temp::Temp;
#[derive(Debug,Clone)]
pub enum BlockType{
    Continuous,
    EndByCondbr,
    EndByBr,
    EndByRet,
}
pub struct BasicBlock{
    pub id:i32,
    pub pred:Vec<i32>,
    pub succ:Vec<i32>,
    pub label:Option<Label>,
    pub range:(i32,i32),
    //存储本basic_block里面使用到的temps的信息
    pub defs:BTreeSet<u32>,
    pub liveuse:BTreeSet<u32>,
    pub livein:BTreeSet<u32>,
    pub liveout:BTreeSet<u32>,
}
impl BasicBlock{
    pub fn new(label:Option<Label>,id:i32,start:i32,end:i32)->BasicBlock{
        BasicBlock{
            id,
            pred:Vec::new(),
            succ:Vec::new(),
            label,
            range:(start,end),
            defs:BTreeSet::new(),
            liveuse:BTreeSet::new(),
            livein:BTreeSet::new(),
            liveout:BTreeSet::new(),
        }
    }
}

