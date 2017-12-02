extern crate kago;

fn main() {
    let res = kago::executor::run();
    println!("Hello, world! #{:?}", res);
}
