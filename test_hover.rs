fn main() {
    let content = "owns_state = 123";
    let script = server::parser::parse_script(content).unwrap();
    println!("{:?}", script.entries[0]);
}
