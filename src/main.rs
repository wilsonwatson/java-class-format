
fn main() {
    let example = std::fs::read("example.class").unwrap();
    let example = java_class_format::ClassFile::parse(example).unwrap();
    for method in example.methods() {
        if let Some(code) = method.code().unwrap() {
            println!("{:#?}", code.instructions().unwrap());
        }
    }
}
