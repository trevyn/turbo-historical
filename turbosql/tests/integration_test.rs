use turbosql::{Blob, Turbosql};

#[derive(Turbosql, Default)]
struct Person {
 rowid: Option<i64>, // rowid member required & enforced at compile time
 name: Option<String>,
 age: Option<i64>,
 image_jpg: Option<Blob>,
}

#[test]
fn it_adds_two() {
 // assert!(1 + 1 == 2);
}
