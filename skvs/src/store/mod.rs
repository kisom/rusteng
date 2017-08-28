//! store implements the backing key-value store for the simple
//! key-value store. At its core, it is a hash map linking a `String`
//! key to an `Entry`.
pub mod entry;

use self::entry::Entry;
use std::collections::HashMap;
use std::string::ToString;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Result contains results for write operations on the SKVS.
pub enum WriteResult {
    /// AlreadyExists is returned when inserting an entry under a key
    /// that already exists. It implies that the insert was rejected.
    AlreadyExists,
    /// Inserted is returned when inserting an entry under a key that
    /// didn't already exist.
    Inserted,
    /// Updated is returned when updating the entry for an
    /// already-existing key.
    Updated,
}

use self::WriteResult::*;

impl ToString for WriteResult {
    fn to_string(&self) -> String {
        match *self {
            AlreadyExists => return "key already exists".to_string(),
            Inserted      => return "new entry inserted".to_string(),
            Updated       => return "updated existing entry".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
/// metrics contains information about the SKVS.
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
    /// new returns initialises an empty Metrics structure.
    pub fn new() -> Metrics {
        Metrics { last_update: 0, last_write: 0, size: 0, write_error: "".to_string() }
    }
}

/// A `Store` is a simple key value store that persists to disk.
pub struct Store {
    /// path is the location on disk of the persisted SKVS.
    pub path: String,

    pub metrics: Metrics,
    pub values: HashMap<String, Entry>,
}

/// `new` returns an empty `Store`.
pub fn new(store_path: String) -> Store {
    Store {
        path: store_path.clone(),
        metrics: Metrics::new(),
        values: HashMap::new(),
    }
}

impl Store {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn insert(&mut self, k: String, v: String) -> WriteResult {
        if self.values.contains_key(&k) {
            AlreadyExists
        } else {
            self.values.insert(k, Entry::from_string(v));
            Inserted
        }                
    }

    fn update(&mut self, k: String, v: String) -> WriteResult {
        panic!("not implemented yet");
        AlreadyExists
    }
}


#[test]
fn test_new() {
    let mut kvs = new("".to_string());
    assert_eq!(kvs.len(), 0);

    let mut wr: WriteResult;
    wr = kvs.insert("X-Pro2".to_string(), "Fujifilm".to_string());
    assert_eq!(wr, Inserted);
    assert_eq!(kvs.len(), 1);

    // Make a mistake.
    wr = kvs.insert("D800".to_string(), "Canon".to_string());
    assert_eq!(wr, Inserted);
    assert_eq!(kvs.len(), 2);

    // Fix it.
    wr = kvs.insert("D800".to_string(), "Nikon".to_string());
    assert_eq!(wr, AlreadyExists);
    assert_eq!(kvs.len(), 2);

    wr = kvs.update("D800".to_string(), "Nikon".to_string());
    assert_eq!(wr, Updated);
    assert_eq!(kvs.len(), 2);    
}

