fn main() {
    hello_world();
    tell_height(182);
    human_id("Daniel", 26, 185.0);
}

// hoisting - 
// can call function anywhere in your code.
fn hello_world(){
    println!("Hello, Rust!");
}

fn tell_height(height: u32){
    println!("My height is {} cm.", height);
}

fn human_id(name: &str, age: u32, height: f32){
    println!("My name is {}, I am {} years old, and my height is {} cm.", name, age, height);
}


