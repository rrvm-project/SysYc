use super::{
	constrain::Constrain, constrain_graph::ConstrainGraph, RangeAnalysis,
};

pub fn solve_graph(
	sccs: Vec<Vec<usize>>,
	mut graph: ConstrainGraph,
) -> ConstrainGraph {
	graph.prepare();
	println!("=================");
	for ssc in sccs.iter().rev() {
		analysis_scc(ssc, &mut graph);
	}
	graph
}

fn analysis_scc(scc: &Vec<usize>, graph: &mut ConstrainGraph) {
	if scc.len() == 1 {
		// assert!(graph.narrowing_node(scc[0]));
		let result = graph.narrowing_node(scc[0]);
		dbg!(result);
	} else {
		todo!()
	}
}
