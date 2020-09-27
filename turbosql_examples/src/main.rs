use turbosql::{select, Blob, Turbosql};

#[derive(Turbosql, Debug, Default)]
struct Person {
 rowid: Option<i64>, // rowid member required & enforced at compile time
 name: Option<String>,
 age: Option<i64>,
 image_jpg: Option<Blob>,
}

fn main() {
 // INSERT single row -- call insert() with rowid: None
 // TODO: is this optional in the declaration, defaulting to None?
 let person = Person {
  rowid: None,
  name: Some("Joe".to_owned()),
  age: None,
  image_jpg: None,
 };
 // let rows_inserted = person.insert().unwrap();
 // println!("rows inserted: {}", rows_inserted);

 // println!(
 //  "{:#?}",
 //  Person::select_where("rowid = 2", turbosql::params![])
 // );
 // println!("{:#?}", Person::select_all());

 // SELECT a person
 let person = select!(Person "WHERE rowid = 2");
 println!("{:#?}", person);
}

// // SELECT multiple rows with a predicate
// let people: Vec<Person> = select!(Vec<Person> "WHERE age > ?", 21)?;

// // SELECT a single row with a predicate
// let person: Person = select!(Person "WHERE name = ?", "Joe")?;

// // UPDATE single row -- call update() with rowid: Some(i64)
// let mut person = select!(Person "WHERE name = ?", "Joe")?;
// person.age = 18;
// person.update()?;

// // UPSERT batch

// let people: Vec<Person> = vec![person, person];
// upsert_batch!(Person, &people)?;
