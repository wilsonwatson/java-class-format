
fn main() {
    let example = std::fs::read("example.class").unwrap();
    let example = java_class_format::ClassFile::parse(example).unwrap();
    println!("{:#?}", example);
}
