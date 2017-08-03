extern crate sass_rs;

fn main() {
    println!("Hello, world!");


    let out = sass_rs::compile_file("resources/main.sass", sass_rs::Options::default());

    println!("out -> {:?}", out);
}
