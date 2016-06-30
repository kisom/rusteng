extern crate getopts;
use getopts::Options;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("a", "", "Address server should listen on.", "ADDRESS");
    opts.optopt("f", "", "Path to disk store.", "FILE");    
    opts.optflag("h", "help", "Print a short usage message.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", args[0]);        
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut addr = "localhost:8000";
    if matches.opt_present("a") {
        match matches.opt_str("a") {
            Some(a) => { addr = a; }
            None    => { panic!("address argument present but unavailable."); }
        };
    }

    let mut store_file = "store.json";
    if matches.opt_present("f") {
        match matches.opt_str("f") {
            Some(f) => { store_file = f; }
            None    => { panic!("store file argument present but unavailable."); }
        };
    }
}
