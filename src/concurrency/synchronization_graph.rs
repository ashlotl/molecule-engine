use std::sync::{
    Arc,
    Mutex,
    RwLock,
};

use crossbeam::{
    crossbeam_channel,
    crossbeam_channel::{
        Receiver,
        Sender,
    }
};

use crate::concurrency::tasks::{
    task::Task,
    task_executor::{
        TaskExecutor,
        TaskExecutorInner,
    },
};

#[derive(Default)]
pub struct SynchronizationGraphTemplate {    
    nodes: Vec<Arc<Mutex<TemplateSynchronizationNode>>>,
    required_by: Vec<Arc<RwLock<dyn Task>>>,
}

impl SynchronizationGraphTemplate {

    pub fn build_and_submit(self, entrypoints: Vec<String>) -> Result<TaskExecutor, String> {
        for node_i in 0..self.nodes.len() {
            let node_ref = self.nodes[node_i].clone();
            let node = node_ref.lock().unwrap();

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
                    let check_node = check_node_ref.lock().unwrap();

                    
                    if node.children[child_i] == check_node.name {
                        let (tx, rx) = crossbeam_channel::bounded(1);
                        if entrypoints.contains(&String::from(check_node.name.clone())) {
                            tx.send(()).unwrap();
                        }
                        let parent_sub_node_lock = node.node.clone();
                        let mut parent_sub_node = parent_sub_node_lock.lock().unwrap();
                        parent_sub_node.children.push(tx);

                        let child_sub_node_lock = check_node.node.clone();
                        let mut child_sub_node = child_sub_node_lock.lock().unwrap();
                        child_sub_node.parents.push(rx);

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
            let dependent_lock = self.required_by[dependent_i].clone();
            let mut dependent = dependent_lock.write().unwrap();
            dependent.submit_nodes(&self.nodes);
            if !dependent.values_filled() {
                let mut list_str = String::from("");
                for name in dependent.submitted_node_list() {
                    list_str.push_str(name.as_str());
                }
                return Err(format!("Dependency {} could not find all necessary nodes. Submitted nodes were: {}", dependent.name(), list_str));
            }
        }

        let task_executor = TaskExecutor {
            inner: TaskExecutorInner::new(self.required_by),
        };

        Ok(task_executor)
    }

    pub fn node_list(&self) -> &Vec<Arc<Mutex<TemplateSynchronizationNode>>> {
        &self.nodes
    }

    pub fn push_dependency(&mut self, requirement_lock:Arc<RwLock<dyn Task>>) -> Result<(), String> {
        {
            let requirement = requirement_lock.read().unwrap();
            for required_i in 0..self.required_by.len() {
                let dependent_lock = self.required_by[required_i].clone();
                let dependent = dependent_lock.read().unwrap();
                if dependent.name()==requirement.name() {
                    return Err(format!("Dependency name {} is used more than once.", requirement.name()));
                }
            }
        }

        self.required_by.push(requirement_lock);
        Ok(())
    }

    pub fn push_node(&mut self, node: TemplateSynchronizationNode) -> Result<(), String> {
        for node_i in 0..self.nodes.len() {
            let check_node_lock = self.nodes[node_i].clone();
            let check_node = check_node_lock.lock().unwrap();
            if check_node.name()==node.name() {
                return Err(format!("Dependency name {} is used more than once.", node.name()));
            }
        }
        self.nodes.push(Arc::new(Mutex::new(node)));
        Ok(())
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
    node: Arc<Mutex<SynchronizationNode>>,
}

impl std::fmt::Debug for TemplateSynchronizationNode {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("TemplateSynchronizationNode")
            .field("name",&self.name)
            .field("children", &self.children)
            .finish()
    }
}

impl TemplateSynchronizationNode {
    pub fn new(name:String) -> Self {
        Self {
            children:vec![],
            name: name,
            node: Arc::new(Mutex::new(SynchronizationNode::default())),//yet to be filled with valid values
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn node(&self) -> Arc<Mutex<SynchronizationNode>> {
        self.node.clone()
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
        println!("{}", list.len());
        for i in (0..list.len()).rev() {
            match self.submit_node(list[i].clone()) {
                SubmissionResult::Success => {
                    println!("Pushing {} resulted in a success", list[i].lock().unwrap().name());
                    self.push_submitted_node_list(list[i].clone().lock().unwrap().name());
                },
                SubmissionResult::NotFound => {},
            }
        }
    }

    fn submitted_node_list(&self) -> &Vec<String>;

    fn values_filled(&self) -> bool;//all necessary nodes have been submitted
}