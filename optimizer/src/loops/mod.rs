use std::collections::HashMap;

use loop_data::LoopData;

mod chain_node;
mod impls;
mod indvar;
mod indvar_extraction;
mod loop_data;
mod loop_simplify;
mod loopinfo;
mod temp_graph;

pub struct HandleLoops {
	loopdatas: HashMap<String, LoopData>,
}
