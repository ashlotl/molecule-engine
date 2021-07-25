use crate::{
    concurrency::tasks::task::Task,
};

pub struct TaskExecutor {
    pub task_list: Vec<Box<dyn Task>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            task_list:vec![],
        }
    }

    pub fn start_task_loops(&mut self) {
        for task in &self.task_list {
            task.run();
        }
    }
}