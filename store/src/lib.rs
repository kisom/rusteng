//! store implements a key-value store with additional metadata about
//! the store.
//!
//! ```
//! extern crate store;
//!
//! // First, initialise a new store.
//! let mut store = store::Store::new();
//!
//! // Next, let's add some keys to the store.
//! if !store.add("key".to_string(), "value".to_string()) {
//!     panic!("Failed to add 'key' to the store.");
//! }
//!
//! if !store.add("something".to_string(), "else".to_string()) {
//!    panic!("Failed to add 'something' to the store.");
//! }
//!
//! // The return value from `Store::add` is mostly useful for
//! // determining if a value was actually stored or updated. In
//! // most cases, this probably won't be a `panic!`able offense.
//! store.add("hello".to_string(), "world".to_string());
//!
//! // The `Value`s in the store contain more than just the string,
//! // though.
//! match store.clone().get("something".to_string()) {
//!     None    => { println!("Nothing found in the store."); }
//!     Some(v) => { println!("Value: {:?}", v); }
//! };
//! // This should print something like:
//! // `Value { timestamp: 1468376899, version: 1, value: "else" }`
//!
//! // The version and timestamp fields are also updated when `add` is
//! // used to update a value.
//! store.add("something".to_string(), "more".to_string());
//! match store.clone().get("something".to_string()) {
//!     None    => { println!("Nothing found in the store."); }
//!     Some(v) => { println!("Value: {:?}", v); }
//! };
//! // This should print something like:
//! // `Value { timestamp: 1468377212, version: 2, value: "more" }`
//!
//! // A value can be removed from the store with a call to `delete`:
//! if !store.delete("something".to_string()) {
//!    panic!("couldn't remove 'something' from the store.");
//! }
//!
//! if store.delete("something".to_string()) {
//!    panic!("The 'something' value should have already been removed.");
//! }
//! ```
extern crate time;

use std::collections::HashMap;

fn timestamp() -> i64 {
    return time::get_time().sec;
}

/// A Value contains some string stored in the key/value store with
// associated metadata.
#[derive(Clone, Debug)]
pub struct Value {
    timestamp: i64,
    version: u64,
    value: String,
}

impl Default for Value {
    fn default() -> Value {
        Value {
            timestamp: timestamp(),
            version: 1,
            value: "".to_string(),
        }
    }
}

impl Value {
    fn update(&self, v: &String) -> Value {
        Value {
            timestamp: timestamp(),
            version: self.version + 1,
            value: (v.clone()),
        }
    }
}

/// A Metrics structure contains information about the key/value store.
#[derive(Clone, Debug)]
pub struct Metrics {
    /// last_update stores the timestamp for the last time the store
    /// was updated; a call to `Store::delete` or `Store::add` will trigger this.
    pub last_update: i64,

    /// last_write stores the timestamp for the last time the store
    /// was written to disk.
    pub last_write: i64,

    /// size stores the current number of keys in the store.
    pub size: usize,

    /// write_error contains a string error message indicating why the
    /// last write failed. If the last write was successful, this
    /// field will be empty.
    pub write_error: String,
}

impl Metrics {
    /// `new` creates a new empty `Metrics` structure.
    pub fn new() -> Metrics {
        Metrics {
            last_update: 0,
            last_write: 0,
            size: 0,
            write_error: "".to_string(),
        }
    }
}

/// A Store contains key/value pairs along with metadata about the
/// store.
#[derive(Clone, Debug)]
pub struct Store {
    /// path contains the disk path for the store.
    pub path: String,

    /// metrics contains metadata about the store.
    pub metrics: Metrics,

    /// values stores the key-value pairs.
    pub values: HashMap<String, Value>,
}

impl Store {
    /// New initialises a new, empty store.
    pub fn new() -> Store {
        Store {
            path: "".to_string(),
            metrics: Metrics::new(),
            values: HashMap::new(),
        }
    }

    /// add should take a string key and value as input. The key
    /// should be updated with the new value, including an updated
    /// timestamp. The store's metrics should also be updated
    /// (last_update and size).
    pub fn add(&mut self, key: String, vs: String) -> bool {
        if vs.len() == 0 {
            return false;
        }

        let ts = timestamp();
        let mut v = Value { ..Default::default() };
        match self.values.get(&key) {
            Some(kval) => {
                v = kval.update(&vs);
                v.timestamp = ts;
            }
            None => {
                v.value = vs;
                v.timestamp = ts;
            }
        };

        if v.version == 1 {
            self.values.remove(&key);
        }

        self.values.insert(key, v);
        self.metrics.last_update = ts;
        self.metrics.size = self.values.len();
        return true;
    }

    /// get returns the Value structure associated with a key, or
    /// None if no Value could be found.
    pub fn get(self, key: String) -> Option<Value> {
        match self.values.get(&key) {
            Some(kval) => {
                let v = Value {
                    timestamp: kval.timestamp.clone(),
                    version: kval.version.clone(),
                    value: kval.value.clone(),
                };
                return Some(v);
            }
            None => {
                return None;
            }
        };
    }

    /// delete removes the Value associated with a key.
    pub fn delete(&mut self, key: String) -> bool {
        if !self.values.contains_key(&key) {
            return false;
        }

        self.values.remove(&key);
        self.metrics.last_update = timestamp();
        self.metrics.size = self.values.len();
        return true;
    }

    /// last_updated returns the timestamp of the last update on the
    /// store.
    pub fn last_updated(self) -> i64 {
        return self.metrics.last_update.clone();
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::thread::sleep;

    #[test]
    fn store_init() {
        let mut store = ::Store::new();

        if store.metrics.last_update.clone() != 0 {
            panic!("store wasn't zero-initialised!");
        }

        if !store.add("a".to_string(), "b".to_string()) {
            panic!("failed to add 'a' to the store.");
        }

        if store.metrics.last_update.clone() == 0 {
            panic!("store wasn't updated");
        }
    }

    #[test]
    fn store_add() {
        let mut store = ::Store::new();
        if !store.add("a".to_string(), "b".to_string()) {
            panic!("failed to add 'a' to the store.");
        }

        if !store.add("a".to_string(), "c".to_string()) {
            panic!("failed to update 'a' in the store.");
        }

        let v: ::Value;
        match store.get("a".to_string()) {
            Some(kval) => {
                v = kval;
            }
            None => {
                panic!("key wasn't found in store");
            }
        };

        if v.version.clone() != 2 {
            panic!("value wasn't updated properly");
        }
    }

    #[test]
    fn store_metrics_timestamp() {
        let mut store = ::Store::new();
        store.add("a".to_string(), "b".to_string());
        sleep(Duration::new(2, 0));
        store.add("c".to_string(), "d".to_string());

        let mut v: ::Value;
        match store.clone().get("a".to_string()) {
            None => {
                panic!("key not found in store");
            }
            Some(kval) => {
                v = kval;
            }
        };

        if store.clone().last_updated() == v.timestamp.clone() {
            panic!("last update should be later than first entry");
        }

        match store.clone().get("c".to_string()) {
            None => {
                panic!("key not found in store");
            }
            Some(kval) => {
                v = kval;
            }
        };

        if store.clone().last_updated() != v.timestamp.clone() {
            panic!("last update should be the same time as the last entry");
        }
    }

    #[test]
    fn store_delete() {
        let mut store = ::Store::new();
        store.add("a".to_string(), "b".to_string());
        store.add("c".to_string(), "d".to_string());

        if store.metrics.size.clone() != 2 {
            panic!("invalid size for store");
        }

        if store.delete("b".to_string()) {
            panic!("shouldn't delete non-extant key");
        }

        if !store.delete("a".to_string()) {
            panic!("failed to delete key");
        }
    }

    #[test]
    fn store_value_update() {
        let mut store = ::Store::new();
        let version: u64;
        let timestamp: i64;

        store.add("a".to_string(), "b".to_string());
        match store.clone().get("a".to_string()) {
            None => {
                panic!("key not found in store");
            }
            Some(kval) => {
                version = kval.version.clone();
                timestamp = kval.timestamp.clone();
            }
        };

        sleep(Duration::new(2, 0));
        store.add("a".to_string(), "c".to_string());
        match store.clone().get("a".to_string()) {
            None => {
                println!("key not found in store");
            }
            Some(kval) => {
                if timestamp >= kval.timestamp {
                    panic!("value should have been updated.");
                }

                if version >= kval.version {
                    panic!("version should have been updated.");
                }
            }
        };
    }
}
