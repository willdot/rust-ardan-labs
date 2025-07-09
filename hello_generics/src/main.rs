fn just_print_it<T: ToString>(x: T) {
    println!("{}", x.to_string());
}

fn main() {
    just_print_it("hello");
    just_print_it(5);
}
