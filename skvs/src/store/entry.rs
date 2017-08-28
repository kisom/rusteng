//! The Entry structure is used as the value in the simple key-value
//! store's hash map.
extern crate time;

use std::thread;
use std::time::Duration;


/// Entry combines metadata with the actual value to be stored.
///
/// The metadata stored in an Entry is current the Unix timestamp of
/// the last write operation (create or update), the version, and the
/// actual string value. Note that versions start at 1 when the
/// structure is first created.
///
/// The `new` or `from_string` static methods should be called to
/// obtain a new `Entry`.
///
/// An example of the use of the `&str` functions:
///
/// ```
/// let old = Entry::new("hello, world");
/// assert_eq!(ent.version, 1);
/// assert_eq!(ent.value, "hello, world");
/// assert!(ent.time > 0);
///
/// let new = Entry::update(&ent1, "goodbye, world");
/// assert_ne!(old.value, new.value);
/// assert_eq!(new.version, old.version + 1);
/// assert!(new.time >= old.time);
/// ```
///
#[derive(Clone, Debug)]
pub struct Entry {
    /// time stores the timestamp from the last write on the entry,
    /// whether that write is creation (version = 1) or modification
    /// (version > 1);
    pub time: i64,

    /// version is incremented on each write to the entry.
    pub version: i64,

    /// value is the current value of the entry.
    pub value: String,
}

impl Entry {
    /// `new` converts value to a String and initialises a new entry
    /// with the current time and a starting version.
    pub fn new(value: &str) -> Entry {
        Entry::from_string(value.to_string())
    }

    /// `from_string` clones the string argument and initialises a new
    /// entry with the current time and a starting version.
    pub fn from_string(s: String) -> Entry {
        Entry {
            time: time::get_time().sec,
            version: 1,
            value: s.clone(),
        }
    }

    /// `update` returns a new entry with the new value, incrementing
    /// the version number if the new value differs from the old
    /// value.
    pub fn update(old: &Entry, nval: &str) -> Entry {
        // TODO: there should be a way to return `old` instead of
        // reconstructing an `Entry`.
        if old.value == nval.to_string() {
            Entry {
                time: old.time,
                version: old.version,
                value: old.value.clone(),
            }
        } else {
            Entry {
                time: time::get_time().sec,
                version: old.version + 1,
                value: nval.to_string(),
            }
        }
    }

    /// `update_from_string` works like update, except it clones the
    /// string argument.
    pub fn update_from_string(old: &Entry, s: String) -> Entry {
        if old.value == s {
            Entry {
                time: old.time,
                version: old.version,
                value: old.value.clone(),
            }
        } else {
            Entry {
                time: time::get_time().sec,
                version: old.version + 1,
                value: s.clone(),
            }
        }
    }
}

#[test]
fn test_new_entry() {
    let ent = Entry::new("hello, world");
    assert_eq!(ent.version, 1);
    assert_eq!(ent.value, "hello, world");
    assert!(ent.time > 0);
}

#[test]
fn test_update_entry() {
    let ent1 = Entry::new("hello, world");
    thread::sleep(Duration::new(1, 0));
    
    let ent2 = Entry::update(&ent1, "goodbye, world");
    assert_ne!(ent1.value, ent2.value);
    assert_eq!(ent2.version, ent1.version + 1);
    assert!(ent2.time >= ent1.time);
}

#[test]
fn test_string_variants() {
    let ent1 = Entry::from_string("hello, world".to_string());
    assert_eq!(ent1.version, 1);
    assert_eq!(ent1.value, "hello, world".to_string());
    assert!(ent1.time > 0);

    thread::sleep(Duration::new(1, 0));
    
    let ent2 = Entry::update_from_string(&ent1, "goodbye, world".to_string());
    assert_ne!(ent1.value, ent2.value);
    assert_eq!(ent2.version, ent1.version + 1);
    assert!(ent2.time >= ent1.time);    
}
