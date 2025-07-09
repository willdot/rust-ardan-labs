trait Animal {
    fn speak(&self);
}

struct Cat;

impl Animal for Cat {
    fn speak(&self) {
        println!("meow");
    }
}

struct Dog;

impl Animal for Dog {
    fn speak(&self) {
        println!("woof");
    }
}

fn main() {
    let cat = Cat;
    cat.speak();

    let dog = Dog;
    dog.speak();

    speak_twice(&cat);
}

fn speak_twice(animal: &impl Animal) {
    animal.speak();
    animal.speak();
}
