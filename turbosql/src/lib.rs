/*! # Turbosql: Easy Data Persistence Layer, backed by SQLite

WORK IN PROGRESS, use at your own risk. :)

Macros for easily persisting Rust `struct`s to an on-disk SQLite database and later retrieving them, optionally based on your own predicates.

```rust
use turbosql::{Turbosql, Blob};

#[derive(Turbosql, Default)]
struct Person {
 rowid: Option<i64>,  // rowid member required & enforced at compile time
 name: Option<String>,
 age: Option<i64>,
 image_jpg: Option<Blob>
}
```

## Design Goals

- API with minimal cognitive complexity and boilerplate
- High performance
- Reliable storage
- Surface the power of SQL — make simple things easy, and complex things possible
- In the spirit of Rust, move as many errors as possible to compile time

### License: MIT OR Apache-2.0
*/

#![allow(unused_imports)]

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use rusqlite::{Connection, OpenFlags, Statement};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

// re-export

#[doc(hidden)]
pub use once_cell::sync::Lazy;
#[doc(hidden)]
pub use rusqlite::{
 params, types::FromSql, types::FromSqlResult, types::ToSql, types::ToSqlOutput, types::Value,
 types::ValueRef, Error, OptionalExtension, Result,
};
#[doc(hidden)]
pub use serde::Serialize;
pub use turbosql_macros::{execute, select, Turbosql};

/// Wrapper for `Vec<u8>` that provides `Read`, `Write` and `Seek` traits.
pub type Blob = Vec<u8>;

// #[derive(Debug)]
// pub struct Blob {
//  table: String,
//  column: String,
//  rowid: i64,
//  len: i64,
//  bytes: Option<Vec<u8>>,
// }

#[derive(Clone, Debug, Deserialize, Default)]
struct MigrationsToml {
 migrations_append_only: Option<Vec<String>>,
 target_schema_autogenerated: Option<String>,
}

struct DbPath {
 path: PathBuf,
 opened: bool,
}

static __DB_PATH: Lazy<Mutex<DbPath>> = Lazy::new(|| {
 let cur_exe = match std::env::current_exe() {
  Ok(path) => match path.file_stem() {
   Some(stem) => Some(stem.to_str().unwrap().to_string()), // TODO: remove unwrap
   None => None,
  },
  Err(_) => None,
 };

 Mutex::new(DbPath {
  path: Path::new(&match cur_exe {
   Some(name) => format!("{}.sqlite", name),
   None => "turbosql.sqlite".to_owned(),
  })
  .to_owned(),
  opened: false,
 })
});

#[doc(hidden)]
pub static __TURBOSQL_DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
 let toml_decoded: MigrationsToml = toml::from_str(include_str!("../../migrations.toml"))
  .expect("Unable to decode embedded migrations.toml");

 let target_migrations = toml_decoded.migrations_append_only.unwrap_or_else(Vec::new);

 let mut db_path = __DB_PATH.lock().unwrap();

 db_path.opened = true;

 // We are handling the mutex, so SQLite can be opened in no-mutex mode; see:
 // http://sqlite.1065341.n5.nabble.com/SQLITE-OPEN-FULLMUTEX-vs-SQLITE-OPEN-NOMUTEX-td104785.html

 let conn = Connection::open_with_flags(
  &db_path.path,
  OpenFlags::SQLITE_OPEN_READ_WRITE
   | OpenFlags::SQLITE_OPEN_CREATE
   | OpenFlags::SQLITE_OPEN_NO_MUTEX,
 )
 .expect("rusqlite::Connection::open_with_flags");

 conn
  .execute_batch(
   r#"
    PRAGMA auto_vacuum=INCREMENTAL;
    PRAGMA journal_mode=WAL;
    PRAGMA wal_autocheckpoint=8000;
    PRAGMA synchronous=NORMAL;
   "#,
  )
  .expect("Execute PRAGMAs");

 let result = conn.query_row(
  "SELECT sql FROM sqlite_master WHERE name = ?",
  params!["turbosql_migrations"],
  |row| {
   let sql: String = row.get(0).unwrap();
   Ok(sql)
  },
 );

 match result {
  Err(rusqlite::Error::QueryReturnedNoRows) => {
   // no migrations table exists yet, create
   conn
    .execute_batch(
     r#"CREATE TABLE turbosql_migrations (rowid INTEGER PRIMARY KEY, migration TEXT NOT NULL)"#,
    )
    .expect("CREATE TABLE turbosql_migrations");
  }
  Err(err) => {
   panic!(err);
  }
  Ok(_) => (),
 }

 let applied_migrations = conn
  .prepare("SELECT migration FROM turbosql_migrations ORDER BY rowid")
  .unwrap()
  .query_map(params![], |row| {
   // let sql: String = row.get(0).unwrap();
   Ok(row.get(0).unwrap())
  })
  .unwrap()
  .map(|x| x.unwrap())
  .collect::<Vec<String>>();

 // println!("applied_migrations is: {:#?}", applied_migrations);
 // println!("target_migrations is: {:#?}", target_migrations);

 // execute migrations

 applied_migrations.iter().zip_longest(&target_migrations).for_each(|item| match item {
  Both(a, b) => assert!(a == b),
  Left(_) => panic!("More migrations are applied than target"),
  Right(migration) => {
   eprintln!("insert -> {:#?}", migration);
   conn.execute(migration, params![]).unwrap();
   conn
    .execute("INSERT INTO turbosql_migrations(migration) VALUES(?)", params![migration])
    .unwrap();
  }
 });

 // TODO: verify schema against target_schema_autogenerated

 //    if sql != create_sql {
 //     println!("{}", sql);
 //     println!("{}", create_sql);
 //     panic!("Turbosql sqlite schema does not match! Delete database file to continue.");
 //    }

 Mutex::new(conn)
});

/// Set the local path and filename where Turbosql will store the underlying SQLite database.
///
/// Must be called before any usage of Turbosql macros or will return an error.
/// (Should actually be a std::path::Path?)
pub fn set_db_path(path: &Path) -> Result<(), anyhow::Error> {
 let mut db_path = __DB_PATH.lock().unwrap();

 if db_path.opened {
  return Err(anyhow::anyhow!("Trying to set path when DB is already opened"));
 }

 db_path.path = path.to_owned();

 Ok(())
}
