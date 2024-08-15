use std::collections::HashMap;

use loop_data::LoopData;

mod chain_node;
mod impls;
mod indvar;
mod indvar_type;
mod indvar_optimize;
mod loop_data;
mod loop_simplify;
mod loopinfo;
mod para;
mod temp_graph;

pub struct HandleLoops {
	loopdatas: HashMap<String, LoopData>,
}
