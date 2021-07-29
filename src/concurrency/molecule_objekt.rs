use std::sync::{
    Arc,
    Mutex,
    RwLock,
};

pub trait MoleculeObjekt: 'static + dyn_clone::DynClone + mopa::Any {
    fn name(&self) -> String;
}
mopafy!(MoleculeObjekt);



pub type InnerObjektList = Vec<Arc<RwLock<dyn MoleculeObjekt>>>;
pub type ObjektList = Arc<Mutex<InnerObjektList>>;

pub fn clone_objekt_in_list<A: MoleculeObjekt>(objekt_list: &InnerObjektList, name:&str) -> Option<Box<A>> {
    for objekt_lock in objekt_list {
        let objekt_lock = objekt_lock.clone();
        let objekt_any = &*objekt_lock.write().unwrap();
        if objekt_any.name()==name {
            let objekt_result = objekt_any.downcast_ref::<A>();
            if let None = objekt_result {
                return None;
            }
            return Some(dyn_clone::clone_box(objekt_result.unwrap()));
        }
    }
    None
}