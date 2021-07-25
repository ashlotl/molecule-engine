use std::sync::{
    Arc,
    Mutex,
};

use crossbeam::{
    crossbeam_channel,
    crossbeam_channel::{
        Receiver,
        Sender,
    }
};

pub struct SynchronizationGraphTemplate {    
    nodes: Vec<Arc<Mutex<TemplateSynchronizationNode>>>,
    required_by: Vec<Box<dyn SynchronizationDependent>>,
}

impl SynchronizationGraphTemplate {

    pub fn build_and_submit(mut self) -> Result<Vec<Box<dyn SynchronizationDependent>>, &'static str> {
        for node_i in 0..self.nodes.len() {
            let node_ref = self.nodes[node_i].clone();
            let mut node = node_ref.lock().unwrap();

            if node.children.len()==0 {
                return Err(format!("Synchronization node {} has no children!", node.name));
            }

            for child_i in 0..node.children.len() {
                let mut found_match = false;
                for node_j in 0..self.nodes.len() {
                    if node_j == node_i {
                        continue;
                    }
                    let check_node_ref = self.nodes[node_j].clone();
                    let mut check_node = check_node_ref.lock().unwrap();

                    
                    if node.children[child_i] == check_node.name {
                        let (tx, rx) = crossbeam_channel::unbounded();
                        check_node.node.parents.push(rx);
                        node.node.children.push(tx);
                        found_match = true;
                        break;
                    }
                }

                if !found_match {
                    return Err(format!("Bad synchronization graph configuration! Match could not be found for {}, child of {}.", node.children[child_i], node.name));
                }
            }
        }

        for dependent_i in 0..self.required_by.len() {
            let dependent = &mut self.required_by[dependent_i];
            dependent.submit_nodes(&self.nodes);
            if !dependent.values_filled() {
                let mut list_str = String::from("");
                for name in dependent.submitted_node_list() {
                    list_str.push_str(name.as_str());
                }
                return Err(format!("Dependency {} could not find all necessary nodes. Submitted nodes were: {}", dependent.name(), list_str));
            }
        }
        Ok(self.required_by)
    }

    pub fn push_dependency(&mut self, requirement:Box<dyn SynchronizationDependent>) {
        self.required_by.push(requirement);
    }

    pub fn push_node(&mut self, node: TemplateSynchronizationNode) {
        self.nodes.push(Arc::new(Mutex::new(node)));
    }
}

#[derive(Default)]
pub struct SynchronizationNode {
    parents: Vec<Receiver<()>>,
    children: Vec<Sender<()>>,
}

impl SynchronizationNode {
    pub fn wait_for_parents(&self) {
        for parent in &self.parents {
            parent.recv().unwrap();
        }
    }

    pub fn release_children(&self) {
        for child in &self.children {
            child.send(()).unwrap();
        }
    }
}

pub struct TemplateSynchronizationNode {
    children: Vec<String>,
    name: String,
    node: SynchronizationNode,
}

impl TemplateSynchronizationNode {
    pub fn new(name:String) -> Self {
        Self {
            children:vec![],
            name: name,
            node: SynchronizationNode::default(),//yet to be filled with valid values
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn push_child(&mut self, identifier: String) {
        self.children.push(identifier);
    }
}

pub enum SubmissionResult {
    Success,
    NotFound,
}

pub trait SynchronizationDependent {
    fn name(&self) -> String;

    fn push_submitted_node_list(&mut self, name:String);

    fn submit_node(&mut self, node: Arc<Mutex<TemplateSynchronizationNode>>) -> SubmissionResult;

    fn submit_nodes(&mut self, list: &Vec<Arc<Mutex<TemplateSynchronizationNode>>>) {
        for i in list.len()-1..=0 {
            match self.submit_node(list[i].clone()) {
                SubmissionResult::Success => {
                    self.push_submitted_node_list(list[i].clone().lock().unwrap().name());
                },
                SubmissionResult::NotFound => {
                    break;
                },
            }
        }
    }

    fn submitted_node_list(&self) -> Vec<String>;

    fn values_filled(&self) -> bool;//all necessary nodes have been submitted
}