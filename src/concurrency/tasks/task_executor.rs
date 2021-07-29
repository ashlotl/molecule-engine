use std::{
    sync::{
        Arc,
        Mutex,
        RwLock,
    },
    thread,
    thread::JoinHandle,
};

use crate::{
    concurrency::{
        molecule_objekt::ObjektList,
        tasks::task::{
            Task,
            TaskControlFlow,
            TaskList,
        },
    },
};

#[derive(Clone)]
pub struct TaskExecutorInner {
    handle_list: Arc<RwLock<Vec<Option<JoinHandle<()>>>>>,
    task_control: Arc<RwLock<TaskControlFlow>>,
    task_list: TaskList,
}

impl TaskExecutorInner {
    pub fn new(task_list: Vec<Arc<RwLock<dyn Task>>>) -> Self {
        Self {
            handle_list: Arc::new(RwLock::new(vec![])),
            task_control: Arc::new(RwLock::new(TaskControlFlow::Continue)),
            task_list: Arc::new(Mutex::new(task_list)),
        }
    }
}

pub struct TaskExecutor {
    pub inner: TaskExecutorInner,
}

impl TaskExecutor {
    pub fn start_task_loops(&mut self, objekt_list:ObjektList) {
        {
            println!("Starting task loops");
            let task_list = self.inner.task_list.lock().unwrap();
            for task_lock in &*task_list {
                let mut task = task_lock.write().unwrap();
                println!("Task initialization");
                task.init(objekt_list.clone());

                std::mem::drop(task);
                let task_lock_2 = task_lock.clone();
                let control_lock = self.inner.task_control.clone();
                let handle = thread::spawn(move || {
                    let mut task = task_lock_2.write().unwrap();
                    loop {
                        let control = control_lock.read().unwrap();
                        if let TaskControlFlow::Continue = *control {

                        } else {
                            return;
                        }

                        let ret_control = task.tick();
                        if let TaskControlFlow::Continue = ret_control {

                        } else {
                            println!("Task exit.");
                            std::mem::drop(control);
                            let mut control = control_lock.write().unwrap();
                            *control = ret_control;
                            return;
                        }
                    }
                });
                let handle_list_lock = self.inner.handle_list.clone();
                let mut handle_list = handle_list_lock.write().unwrap();
                handle_list.push(Some(handle));
            }
            let handle_list_lock = self.inner.handle_list.clone();
            let mut handle_list = handle_list_lock.write().unwrap();
            for handle in &mut*handle_list {
                handle.take().unwrap().join().expect("Error joining thread, possibly triggered by task/synchronization rebuild.");
            }
        }
        let task_control_lock = self.inner.task_control.clone();
        let task_control = task_control_lock.read().unwrap();
        match &*task_control {
            TaskControlFlow::Continue => panic!("Task control is Continue, but threads have stopped."),
            TaskControlFlow::RebuildTasks(_) => {
                let task_control_lock = self.inner.task_control.clone();
                let task_control = (*task_control_lock.write().unwrap()).clone();
                self.inner = if let TaskControlFlow::RebuildTasks(ret) = task_control {ret} else {unreachable!()};
            }
            TaskControlFlow::Stop(stop_msg) => {
                println!("All tasks stopping with msg: {}", stop_msg);
                return;
            }
        }
    }
}