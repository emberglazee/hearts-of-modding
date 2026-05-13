fn main() {
    let content = "name = DBUG_show_lar_decisions";
    let script = server::parser::parse_script(content).unwrap();
    println!("{:?}", script);
}
