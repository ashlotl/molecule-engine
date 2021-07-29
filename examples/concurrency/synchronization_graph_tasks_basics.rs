use std::sync::{
    Arc,
    Mutex,
    RwLock,
};

use molecule_engine::concurrency::{
    molecule_objekt::{
        clone_objekt_in_list,
        MoleculeObjekt,
        ObjektList,
    },
    tasks::task::{
        Task,
        TaskControlFlow,
    },
    synchronization_graph::{
        SubmissionResult,
        SynchronizationDependent,
        SynchronizationGraphTemplate,
        SynchronizationNode,
        TemplateSynchronizationNode,
    },
};

static FLIP_BREAK_NECK:&'static str = "flip_break_neck";
static FLIP_PREPARE:&'static str = "flip_prepare";
static FLOP_READ_NIETZSCHE:&'static str = "flop_read_nietzsche";

#[derive(Clone)]
struct U32Objekt {
    name: String,
    some_val: Arc<RwLock<u32>>,
}

impl MoleculeObjekt for U32Objekt {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone)]
struct DoAFlip {
    name: String,

    node_a: Option<Arc<Mutex<SynchronizationNode>>>,
    node_b: Option<Arc<Mutex<SynchronizationNode>>>,

    submitted_node_list: Vec<String>,

    u32_ob: Option<Box<U32Objekt>>,
}

impl SynchronizationDependent for DoAFlip {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn push_submitted_node_list(&mut self, name: String) {
        self.submitted_node_list.push(name);
    }

    fn submit_node(&mut self, node_lock: Arc<Mutex<TemplateSynchronizationNode>>) -> SubmissionResult {
        let node = node_lock.lock().unwrap();
        let name_string = node.name();
        let name = name_string.as_str();
        if name == FLIP_PREPARE {
            self.node_a = Some(node.node());
        } else if name == FLIP_BREAK_NECK {
            self.node_b = Some(node.node());
        } else {
            return SubmissionResult::NotFound;
        }
        SubmissionResult::Success
    }

    fn submitted_node_list(&self) -> &Vec<String> {
        &self.submitted_node_list
    }

    fn values_filled(&self) -> bool {
        match (&self.node_a, &self.node_b) {
            (Some(_), Some(_)) => true,
            _ => false
        }
    }
}

impl Task for DoAFlip {
    fn init(&mut self, objekt_list_lock: ObjektList) {
        println!("starting flip init");
        let objekt_list = objekt_list_lock.lock().unwrap();
        let u32_ob = clone_objekt_in_list(&objekt_list, "ernie");
        self.u32_ob = u32_ob;
        if let Some(_) = self.u32_ob {
            println!("Flip got objekt");
        }
        println!("Flipper Initialized");
    }

    fn tick(&mut self) -> TaskControlFlow {
        self.node_a.as_ref().unwrap().lock().unwrap().wait_for_parents();
        self.node_b.as_ref().unwrap().lock().unwrap().wait_for_parents();
        if let Some(u32_ob) = &self.u32_ob {
            let mut u32_val = u32_ob.some_val.write().unwrap();
            if (*u32_val)!=1 {
                *u32_val/=2;
                println!("flip: {}", *u32_val);
            }
        }
        self.node_a.as_ref().unwrap().lock().unwrap().release_children();
        self.node_b.as_ref().unwrap().lock().unwrap().release_children();
        TaskControlFlow::Continue
    }
}

#[derive(Clone)]
struct DoAFlop {
    name: String,

    node_a: Option<Arc<Mutex<SynchronizationNode>>>,

    submitted_node_list: Vec<String>,

    u32_ob: Option<Box<U32Objekt>>,
}

impl SynchronizationDependent for DoAFlop {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn push_submitted_node_list(&mut self, name: String) {
        self.submitted_node_list.push(name);
    }

    fn submit_node(&mut self, node_lock: Arc<Mutex<TemplateSynchronizationNode>>) -> SubmissionResult {
        let node = node_lock.lock().unwrap();
        let name_string = node.name();
        let name = name_string.as_str();
        if name == FLOP_READ_NIETZSCHE {
            self.node_a = Some(node.node());
        } else {
            return SubmissionResult::NotFound;
        }
        SubmissionResult::Success
    }

    fn submitted_node_list(&self) -> &Vec<String> {
        &self.submitted_node_list
    }

    fn values_filled(&self) -> bool {
        match &self.node_a {
            Some(_) => true,
            _ => false
        }
    }
}

impl Task for DoAFlop {
    fn init(&mut self, objekt_list_lock: ObjektList) {
        let objekt_list = objekt_list_lock.lock().unwrap();
        self.u32_ob = clone_objekt_in_list(&*objekt_list, "ernie");
        if let Some(_) = self.u32_ob {
            println!("Flop got objekt");
        }
        println!("Flop initialized");
    }

    fn tick(&mut self) -> TaskControlFlow {
        self.node_a.as_ref().unwrap().lock().unwrap().wait_for_parents();
        if let Some(ob) = &self.u32_ob {
            let mut ob_val = ob.some_val.write().unwrap();
            if (*ob_val)!=1 {
                *ob_val=*ob_val+1;
                println!("flop: {}", ob_val);
            }
        }
        self.node_a.as_ref().unwrap().lock().unwrap().release_children();
        TaskControlFlow::Continue
    }
}

#[allow(dead_code)]
pub fn sync_test() {

    let mut graph = SynchronizationGraphTemplate::default();

    let flip_task = DoAFlip {
        name: String::from("flipper"),
        node_a: None,
        node_b: None,
        submitted_node_list: vec![],
        u32_ob: None,
    };

    let flop_task = DoAFlop {
        name: String::from("flopper"),
        node_a: None,
        submitted_node_list: vec![],
        u32_ob: None,
    };

    match graph.push_dependency(Arc::new(RwLock::new(flip_task))) {
        Ok(_) => {},
        Err(msg) => panic!("{}", msg),//realistically you would simply send the user an error for having done something stupid
    }

    match graph.push_dependency(Arc::new(RwLock::new(flop_task))) {
        Ok(_) => {},
        Err(msg) => panic!("{}", msg),
    }

    let mut tsn = TemplateSynchronizationNode::new(String::from("flip_prepare"));

    tsn.push_child(String::from("flop_read_nietzsche"));
    
    match graph.push_node(tsn) {
        Ok(_) => {},
        Err(msg) => panic!("{}", msg),
    }

    let mut tsn = TemplateSynchronizationNode::new(String::from("flip_break_neck"));

    tsn.push_child(String::from("flop_read_nietzsche"));
    
    match graph.push_node(tsn) {
        Ok(_) => {},
        Err(msg) => panic!("{}", msg),
    }

    let mut tsn = TemplateSynchronizationNode::new(String::from("flop_read_nietzsche"));

    tsn.push_child(String::from("flip_prepare"));
    tsn.push_child(String::from("flip_break_neck"));
    
    match graph.push_node(tsn) {
        Ok(_) => {},
        Err(msg) => panic!("{}", msg),
    }

    println!("Graph now has nodes: ");
    for node in graph.node_list() {
        let node_lock = node.clone();
        let node = node_lock.lock().unwrap();
        println!("{:?}", node);
    }
    println!("------\n");

    let entrypoints = vec![
        String::from("flip_prepare"),
        String::from("flip_break_neck"),
    ];

    let mut executor = match graph.build_and_submit(entrypoints) {
        Ok(task_list) => {task_list},
        Err(msg) => panic!("{}", msg),
    };

    let objekt_list:Arc<Mutex<Vec<Arc<RwLock<(dyn MoleculeObjekt + 'static)>>>>> = Arc::new(Mutex::new(vec![
        Arc::new(RwLock::new(U32Objekt {
            name: String::from("ernie"),
            some_val: Arc::new(RwLock::new(11)),
        })),
    ]));

    executor.start_task_loops(objekt_list);
    
}