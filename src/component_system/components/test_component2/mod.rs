use opencdd_macros::*;

trait Component2 {

}
trait Command {

}

struct Test {
    commands: std::collections::HashMap<String, Box<dyn Command>>
}

#[commands]
impl Test {
    
    #[command]
    fn test(&self) {
        println!("test");
    }

    fn test2(&self) {
        println!("test2");
    }
}
