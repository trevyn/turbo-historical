# turbosql

## Turbosql: Easy Data Persistence Layer, backed by SQLite

WORK IN PROGRESS, use at your own risk. :)

Macros for easily persisting Rust `struct`s to an on-disk SQLite database and later retrieving them, optionally based on your own predicates.

```rust
use turbosql::Turbosql;

[derive(Turbosql)]
struct Person {
 rowid: Option<i64>,  // rowid member required & enforced at compile time
 name: String,
 age: Option<i64>,
 image_jpg: Option<Blob>
}

fn main() {}
```

### Design Goals

- API with minimal cognitive complexity and boilerplate
- High performance
- Reliable storage
- Surface the power of SQL — make simple things easy, and complex things possible
- In the spirit of Rust, move as many errors as possible to compile time

### License: MIT OR Apache-2.0
