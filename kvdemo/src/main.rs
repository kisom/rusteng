extern crate getopts;
extern crate time;

use getopts::Options;
use std::env;

fn timestamp() -> i64 {
    return time::get_time().sec;
}

struct Value {
    timestamp: i64,
    version:   u64,
    value:     String
}

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

    let mut addr: String  = "localhost:8000".to_string();
    if matches.opt_present("a") {
        match matches.opt_str("a") {
            Some(a) => { addr = a; }
            None    => { panic!("address argument present but unavailable."); }
        };
    }

    let mut store_file: String = "store.json".to_string();
    if matches.opt_present("f") {
        match matches.opt_str("f") {
            Some(f) => { store_file = f; }
            None    => { panic!("store file argument present but unavailable."); }
        };
    }

    println!("started at {}", timestamp());
    println!("listening on {}", addr);    
}
