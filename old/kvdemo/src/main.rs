extern crate getopts;
extern crate time;

use getopts::Options;
use std::collections::HashMap;
use std::env;

fn timestamp() -> i64 {
    return time::get_time().sec;
}

// A Value contains some string stored in the key/value store with
// associated metadata.
#[derive(Debug)]
struct Value {
    timestamp: i64,
    version:   u64,
    value:     String
}

// A Metrics structure contains information about the key/value store.
#[derive(Debug)]
struct Metrics {
    last_update: i64,
    last_write:  i64,
    size:        u64,
    write_error: String
}

// A Store contains key/value pairs along with metadata about the
// store.
#[derive(Debug)]
struct Store {
    // The path to the disk file for the store.
    path: String,

    metrics: Metrics,
    values: HashMap<String, Value>
}

impl Store {
    fn add(&mut self, key: String, vs: String) -> bool {
        let mut v: Value;
        
        // Empty strings aren't valid in this store.
        if vs.is_empty() {
            return false;
        }

        match self.values.get(&(key.clone())) {
            Some(value) => {
                if value.value == vs {
                    return false;
                }
                v = *value;
            }
            _           => {}
        }

        v.timestamp = timestamp();
        v.version += 1;
        v.value = vs;

        self.values.insert(key, v);
        return true;
    }

    fn get(&self, key: String) -> Option<Value> {
        match self.values.get(&(key.clone())) {
            Some(v) => { return Some(*v); }
            None    => { return None; }
        };
    }
}

fn main() {
    let mut store: Store;
    store.values = HashMap::new();
    store.path = "store.json".to_string();
    
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

    if matches.opt_present("f") {
        match matches.opt_str("f") {
            Some(f) => { store.path = f; }
            None    => { panic!("store file argument present but unavailable."); }
        };
    }

    if !store.add("test key".to_string(), "test value".to_string()) {
        panic!("at the disco");
    }
    
    println!("started at {}", timestamp());
    println!("listening on {}", addr);
}
