use cillio_graph::Graph;
use petgraph::algo::toposort;
use serde_json::Value;
use wasmtime::component::Val;

use crate::Runtime;

#[derive(Debug)]
struct ExecutionStep<S> {
    node_id: usize,
    node_type: String,
    node_state: Option<S>,
}

impl<S: std::fmt::Debug> ExecutionStep<S> {
    async fn execute(&self, runtime: &mut Runtime) -> Result<(), anyhow::Error> {
        println!(
            "Executing node {} - {}: {:?}",
            self.node_id, self.node_type, self.node_state,
        );
        let instance = runtime
            .initialize_node(self.node_type.as_str(), Some(&self.node_state))
            .await?;

        let run_fn_name = "process";

        let run_fn = instance
            .get_func(&mut runtime.store, run_fn_name)
            .ok_or(anyhow::anyhow!("Function not found"))?;

        let fn_params = run_fn.params(&runtime.store);
        println!("Function params: {:?}", fn_params);
        let params = [];
        let mut returns = [Val::Record(Vec::new())];
        run_fn
            .call_async(&mut runtime.store, &params, &mut returns)
            .await?;
        println!("Returns: {:?}", returns);

        // let params = runtime.get_params_data(&self.node_id).unwrap();
        // let results = runtime.get_results_data(&self.node_id).unwrap();
        // run_fn
        //     .call_async(&mut runtime.store, params, results)
        //     .await?;

        //let run_fn = run_fn.typed::<(), ({ number: f32})>(&mut runtime.store)?;

        // let results = run_fn.call_async(&mut runtime.store, ()).await?;
        //println!("Results: {:?}", results);

        // let run_fn = module
        //     .get_export(&run_fn_name)
        //     .
        //     .ok_or(anyhow::anyhow!(format!(
        //         "Function <{}> for <{}> not found",
        //         run_fn_name, self.node_type
        //     )))?;
        // let params = run_fn.params(&runtime.store);
        // let results = run_fn.results(&runtime.store);
        // println!("Run function: {:?} - {:?}", &params, &results);
        //run_fn.call_async(&mut runtime.store, , &[]);
        // let run_fn = instance
        //     .get_typed_func::<(), f32>(&mut runtime.store, "run")
        //     .expect("Function not found");
        //run_fn.call_async(&mut runtime.store, ()).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExecutionPlan {
    steps: Vec<ExecutionStep<Value>>,
}

impl ExecutionPlan {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn from_graph(graph: &Graph) -> Self {
        let digraph = graph.graph();
        let sorted_nodes = toposort(digraph, None).expect("The graph is cyclic!");
        let execution_steps: Vec<_> = sorted_nodes
            .into_iter()
            .map(|node_index| {
                let node = digraph.node_weight(node_index).unwrap();
                ExecutionStep {
                    node_id: node_index.index(),
                    node_type: node.data().r#type.clone(),
                    node_state: node.data().state.clone(),
                }
            })
            .collect();
        Self {
            steps: execution_steps,
        }
    }

    pub async fn execute(&self, runtime: &mut Runtime) -> Result<Vec<()>, anyhow::Error> {
        let store_data = runtime.get_store().data();
        println!("Execute: {:?}", store_data);
        for step in &self.steps {
            step.execute(runtime).await?;
        }
        Ok(vec![])
    }
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        ExecutionPlan::new()
    }
}
