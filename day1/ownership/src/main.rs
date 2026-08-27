//Ownership, Borrowing and References

//Ownership
// C, C++ -> Memory Management Control Issue
// Garbage Colelctor solved this issue, but created a new issue -> slow performance:
// Stopping/resuming the application.

//In Rust:
// 1. Each value in Rust has a variable that's its Owner.
// 2. There can be only one owner at a time.
// 3. When the owner goes out of scope, the value will be dropped.
//
// Example of 1:
fn main(){
    let s1 = String::from("RUST"); // s1 is the owner!
    let len = calculate_length(&s1); //calculate_length borrows a reference to the value.
    println!("Length of '{}' is {}.", s1, len);

    //Example of 2:
    // This example will fail because s1 is no longer the owner of the value, so there is no value
    // under s1.

    //let s2 = s1;
    //println!("{}", s1);
    

    //next lesson function!
    borrow_function();    
}

fn calculate_length(s: &String) -> usize{
    s.len()
}

//References and Borrowing
//Safety and performance
//Borrowing and references are powerful concepts!
fn borrow_function(){
    println!();
    
    //references allow us to borrow.
    //These references can be mutable and not mutable!
    //We create a reference by using &!
    
    let mut x: i32 = 5;
    let r: &mut i32 = &mut x;
    
    *r += 1;

    println!("{}",x);
    
}

