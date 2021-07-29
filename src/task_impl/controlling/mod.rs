use std::sync::{
    Arc,
    Mutex,
};

use crate::concurrency::{
    molecule_objekt::ObjektList,
    synchronization_graph::{
        SubmissionResult,
        SynchronizationDependent,
        TemplateSynchronizationNode,
    },
    tasks::task::{
        Task,
        TaskControlFlow,
    },
};

#[derive(Clone)]
pub struct Controlling {
    pub name: String,
    pub submitted_node_list: Vec<String>,
}

impl SynchronizationDependent for Controlling {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn push_submitted_node_list(&mut self, name:String) {
        self.submitted_node_list.push(name);
    }

    fn submit_node(&mut self, _node: Arc<Mutex<TemplateSynchronizationNode>>) -> SubmissionResult {
        SubmissionResult::NotFound
    }

    fn submitted_node_list(&self) -> &Vec<String> {
        &self.submitted_node_list
    }

    fn values_filled(&self) -> bool {
        true
    }
}

impl Task for Controlling {
    fn init(&mut self, _objekt_list_lock: ObjektList) {
        unimplemented!();
    }
    
    fn tick(&mut self) -> TaskControlFlow {
        unimplemented!();
    }
}