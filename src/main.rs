extern crate kago;
extern crate sloggers;

use sloggers::Build;
use sloggers::terminal::{TerminalLoggerBuilder, Destination};
use sloggers::types::Severity;

fn main() {
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stdout);
    let logger = builder.build().unwrap();

    let res = kago::executor::run(logger);
    println!("Hello, world! #{:?}", res);
}
