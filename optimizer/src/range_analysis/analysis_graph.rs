use super::constrain_graph::ConstrainGraph;

pub fn solve_graph(
	sccs: Vec<Vec<usize>>,
	mut graph: ConstrainGraph,
) -> ConstrainGraph {
	graph.prepare();

	for ssc in sccs.iter().rev() {
		analysis_scc(ssc, &mut graph);
	}
	graph
}

fn analysis_scc(scc: &Vec<usize>, graph: &mut ConstrainGraph) {
	graph.grow_analysis(scc);
	graph.solve_future(scc);
	graph.narrowing(scc);
}
