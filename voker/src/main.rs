
extern crate voker;


pub fn main() {
//    voker::run_samples();
    voker::command::run_docopt();


}

//pub fn recurser_cosumption() {
    //    let c = |d:&str| { println!("directive {:?}", d) };
    //    recurser(16, &c );
//}


//pub fn recurser<F>(n: u8, f:&F) -> u8 where F: Fn(&str) -> () {
//    f("what");
//    if n == 0 {
//        n
//    } else {
//        recurser(n - 1, f)
//    }
//}