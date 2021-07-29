use std::{
    sync::{
        Arc,
        Mutex,
        RwLock,
    },
};

use crate::concurrency::{
    molecule_objekt::{
        ObjektList,
    },
    synchronization_graph::SynchronizationDependent,
    tasks::task_executor::TaskExecutorInner,
};


pub type InnerTaskList = Vec<Arc<RwLock<dyn Task>>>;
pub type TaskList = Arc<Mutex<InnerTaskList>>;

#[derive(Clone)]
pub enum TaskControlFlow {
    Continue,
    RebuildTasks(TaskExecutorInner),
    Stop(String),
}

pub trait Task: 'static + SynchronizationDependent + Send + Sync {
    fn init(&mut self, objekt_list:ObjektList);
    fn tick(&mut self) -> TaskControlFlow;
}