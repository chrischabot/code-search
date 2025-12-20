
fn main() {
    println!("Hello, World!");
}

pub fn calculate(x: i32, y: i32) -> i32 {
    x + y
}

pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn add(&mut self, x: i32) -> &mut Self {
        self.value += x;
        self
    }
}
