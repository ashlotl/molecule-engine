use take_for_scope::take_for_scope;

#[allow(dead_code)]
pub fn demonstrate_take_for_scope() {
    let mut value = Some(5);
    {
        let value1 = &mut value;
        let took = take_for_scope!(value1, i32).0.unwrap();
        println!("{}", took);
        assert_eq!(value, Some(took));
    }
    
    println!("{}", value.take().unwrap());
}