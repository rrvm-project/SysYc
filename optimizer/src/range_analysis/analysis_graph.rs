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
	dbg!(&graph);
	graph.grow_analysis(scc);
	dbg!(&graph);
	graph.solve_future(scc);
	dbg!(&graph);
	graph.narrowing(scc);
	dbg!(&graph);
}
