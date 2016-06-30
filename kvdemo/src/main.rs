extern crate getopts;
use getopts::Options;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("a", "", "Address server should listen on.", "ADDRESS");
    opts.optflag("h", "help", "Print a short usage message.");
    opts.optopt("f", "", "Path to disk store.", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} FILE [options]", args[0]);        
        print!("{}", opts.usage(&brief));
        return;
    }
}
