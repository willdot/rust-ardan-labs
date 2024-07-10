fn main() {
    println!("mutable");
    // Mutable
    let mut n: i32 = 5;
    n += 1;
    println!("{n}");

    println!("shadowing");
    // Shadowing - the n declared above is a different one but no longer accessable
    let n = 5;
    let n = n + 2;
    println!("{n}");

    println!("scoping");
    // This puts a new one inside the scope of the {} so n is only available within the scope
    // so as soon as the scope is closed, it's no longer accessable
    {
        let n = 6;
        println!("{n}");
    }

    println!("returning from a scope");
    // what ever the last thing declared in the scope, will be returned
    let n: i32 = {
        println!("this is within a scope");
        99
    };
    println!("{n}");

    println!("print result from a function");
    let n: i32 = double(55);
    println!("{n}");

    println!("{}", double_or_nothing(88));

    println!("passing variables to functions and ownership");
    let name: String = "hello".to_string();
    // When you pass a variable to a function, you pass ownership to that function.
    // This means you can no longer use the variable here. So to be able to use it
    // you have to return it from the function and shadow it
    let name: String = greet(name);
    //What you can do though is clone it, so you send a copy of it to a function. This isn't the
    // best choice though as it can be slow
    greet(name.clone());
    greet(name);

    // Or you can "borrow" which is like passing a reference to the variable to the function
    // (like passing a pointer in Go), however you cannot mutate the value.
    let borrowed_name: String = "borrowed_hello".to_string();
    greet_borrow(&borrowed_name);
    greet_borrow(&borrowed_name);

    // to mutate the value you have to pass in a mutable value and the function has to accept a mutable value
    let mut mutable_name: String = "mutable_string".to_string();
    greet_borrow_mut(&mut mutable_name);
    println!("{mutable_name}");

    println!("reading input");

    let input = read_line();
    println!("you type: [{input}]");
}

fn read_line() -> String {
    let mut input: String = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("stdin not working");
    input.trim().to_string()
}

fn greet_borrow_mut(s: &mut String) {
    *s = format!("Why hello there {s}")
}

fn greet_borrow(s: &String) {
    println!("within the borrowing function");
    println!("Hello {s}");
}

fn greet(s: String) -> String {
    println!("Hello {s}");
    s
}

fn double(n: i32) -> i32 {
    println!("within a function");
    // NOTE: you don't need to use the return keyword here. The last thing
    // declared will be returned
    n * 2
}

fn double_or_nothing(n: i32) -> i32 {
    println!("within the weird branch logic");
    if n > 0 {
        n * 2
    } else {
        0
    }

    // NOTE: since we didn't return in the else branches, the last thing
    // declared will be returned, so that's either the n*2 or 0
}
