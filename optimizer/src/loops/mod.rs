use std::collections::HashMap;

use loop_data::LoopData;

mod add_value_to_cfg;
mod chain_node;
mod impls;
mod indvar;
mod indvar_combine;
mod indvar_extraction;
mod indvar_type;
mod loop_data;
mod loop_simplify;
mod loop_unroll;
mod loopinfo;
mod para;
mod temp_graph;

pub struct HandleLoops {
	loopdatas: HashMap<String, LoopData>,
}
