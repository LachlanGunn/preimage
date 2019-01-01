extern crate data_encoding;
extern crate lmdb;

use self::lmdb::Cursor;
use self::lmdb::Transaction;

#[derive(Debug)]
pub enum LookupError {
    NotFound,
    LookupFailed,
}

impl ::std::fmt::Display for LookupError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                LookupError::NotFound => "Key not found",
                LookupError::LookupFailed => "Lookup failed",
            }
        )
    }
}

impl ::std::error::Error for LookupError {}

pub trait ObjectSink {
    fn push(&self, object: &crate::objects::Object);
    fn lookup(&self, key: &[u8]) -> Result<String, Box<::std::error::Error>>;
}

#[allow(dead_code)]
pub struct DebugSink {}

#[allow(dead_code)]
impl DebugSink {
    pub fn new(_app: &crate::config::PreimageApp) -> Result<Self, Box<::std::error::Error>> {
        Ok(DebugSink {})
    }
}

impl ObjectSink for DebugSink {
    fn push(&self, object: &crate::objects::Object) {
        let hashes = match object.hash() {
            Ok(h) => h,
            Err(_) => return,
        };

        for hash in hashes {
            println!(
                "{}: {}",
                data_encoding::HEXLOWER.encode(&hash),
                object.to_str()
            );
        }
    }

    fn lookup(&self, _: &[u8]) -> Result<String, Box<::std::error::Error>> {
        Err(Box::new(LookupError::LookupFailed))
    }
}

pub struct LMDBSink {
    env: lmdb::Environment,
    db: lmdb::Database,
}

impl LMDBSink {
    pub fn new(app: &crate::config::PreimageApp) -> Result<LMDBSink, Box<::std::error::Error>> {
        let env_path = app.path.join(::std::path::PathBuf::from("db"));
        let environment = lmdb::Environment::new()
            .set_map_size(0x4000_0000) // 1GiB
            .open(&env_path)?;
        let database = environment.create_db(None, lmdb::DatabaseFlags::default())?;
        Ok(LMDBSink {
            env: environment,
            db: database,
        })
    }
}

impl ObjectSink for LMDBSink {
    fn push(&self, object: &crate::objects::Object) {
        let mut tx = match self.env.begin_rw_txn() {
            Ok(tx) => tx,
            Err(_) => return,
        };

        let hashes = match object.hash() {
            Ok(hashes) => hashes,
            Err(_) => return,
        };

        let location = object.to_str().into_bytes();

        for hash in hashes {
            {
                let mut cursor = tx.open_ro_cursor(self.db).expect("Failed to get cursor.");
                if let Ok(iterator) = cursor.iter_dup_of(&&hash[..]) {
                    for (_key, value) in iterator {
                        if value[..] == location[..] {
                            return;
                        }
                    }
                }
            }
            tx.put(
                self.db,
                &&hash[..],
                &&location[..],
                lmdb::WriteFlags::default(),
            )
            .expect("Database write failed.");
        }
        tx.commit().expect("Database commit failed.");
    }

    fn lookup(&self, key: &[u8]) -> Result<String, Box<::std::error::Error>> {
        let tx = self.env.begin_ro_txn()?;
        let result = match tx.get(self.db, &key) {
            Ok(v) => v,
            Err(lmdb::Error::NotFound) => return Err(Box::new(LookupError::NotFound)),
            Err(_) => return Err(Box::new(LookupError::LookupFailed)),
        };
        Ok(String::from_utf8(result.to_vec())?)
    }
}
