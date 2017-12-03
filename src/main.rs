extern crate kago;
extern crate simplelog;

use simplelog::{Config, SimpleLogger, LogLevelFilter};

fn main() {
    let _ = SimpleLogger::init(LogLevelFilter::Debug, Config::default()).unwrap();

    let res = kago::executor::run();
    println!("Hello, world! #{:?}", res);
}
