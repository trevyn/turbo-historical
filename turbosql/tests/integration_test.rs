use turbosql::{select, Blob, Turbosql};

#[derive(Turbosql, Default, Debug)]
struct Person {
 rowid: Option<i64>,
 name: Option<String>,
 age: Option<i64>,
 image_jpg: Option<Blob>,
}

#[test]
fn it_works() {
 Person { rowid: None, name: Some("Bob".to_string()), age: Some(42), image_jpg: None }
  .insert()
  .unwrap();

 println!("{:#?}", select!(Vec<Person>));
}
