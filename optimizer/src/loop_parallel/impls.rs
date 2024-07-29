use std::collections::{HashMap, HashSet, VecDeque};

use super::LoopParallel;
use crate::RrvmOptimizer;
use rrvm::{func::Entrance, program::LlvmProgram};
use utils::errors::Result;

fn process_func(func: &mut LlvmFunc) -> bool{
    
    false
}


impl RrvmOptimizer for LoopParallel{
    fn new() -> Self {
        Self{

        }
    }

    fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
        Ok(program.funcs.iter_mut().map(process_func).any())
    }
}