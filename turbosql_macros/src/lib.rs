//! This crate provides Turbosql's procedural macros.
//!
//! ```edition2018
//! # use turbosql::Turbosql;
//! #
//! #[derive(Turbosql)]
//! # struct S;
//! #
//! # fn main() {}
//! ```
//!
//! Please refer to the `turbosql` crate for how to set this up.

// #![allow(unused_imports)]
const SQLITE_64BIT_ERROR: &str = r##"Sadly, SQLite cannot natively store unsigned 64-bit integers, so TurboSQL does not support u64 members. Use i64, u32, f64, or a string or binary format instead. (see https://sqlite.org/fileformat.html#record_format )"##;

use once_cell::sync::Lazy;
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use rusqlite::{params, Connection, Statement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
 parse_macro_input, Data, DeriveInput, Expr, Fields, FieldsNamed, Ident, LitStr, Meta, NestedMeta,
 Token, Type,
};

const MIGRATIONS_FILENAME: &str = "migrations.toml";

mod create;
mod insert;
mod select;

struct Table {
 ident: Ident,
 span: Span,
 name: String,
 columns: Vec<Column>,
}

#[derive(Debug)]
struct MiniTable {
 name: String,
 columns: Vec<MiniColumn>,
}

impl ToTokens for Table {
 fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
  let ident = &self.ident;
  tokens.extend(quote!(#ident));
 }
}

#[derive(Debug)]
struct Column {
 ident: Ident,
 span: Span,
 name: String,
 sqltype: &'static str,
}

#[derive(Debug)]
struct MiniColumn {
 name: String,
 sqltype: &'static str,
}

// static TEST_DB: Lazy<Mutex<Connection>> =
//  Lazy::new(|| Mutex::new(Connection::open_in_memory().unwrap()));

static LAST_TABLE_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("none".to_string()));

static TABLES: Lazy<Mutex<HashMap<String, MiniTable>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// #[proc_macro]
// pub fn set_db_path(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//  let input = proc_macro2::TokenStream::from(input);

//  eprintln!("IN SET DB PATH!");
//  eprintln!("{:#?}", input);

//  let mut db_path = DB_PATH.lock().unwrap();

//  let mut iter = input.into_iter();

//  *db_path = match iter.next() {
//   Some(proc_macro2::TokenTree::Literal(literal)) => literal.to_string(),
//   _ => panic!("Expected string literal"),
//  };

//  proc_macro::TokenStream::new()
// }

#[derive(Debug)]
struct QueryParams {
 params: Punctuated<Expr, Token![,]>,
}

#[derive(Debug)]
struct SelectTokens {
 tokens: proc_macro2::TokenStream,
}

#[derive(Debug)]
struct SelectCTETokens {
 tokens: proc_macro2::TokenStream,
}

#[derive(Debug)]
struct ExecuteTokens {
 tokens: proc_macro2::TokenStream,
}

impl Parse for QueryParams {
 fn parse(input: ParseStream) -> syn::Result<Self> {
  Ok(QueryParams {
   params: if input.peek(Token![,]) {
    input.parse::<Token![,]>().unwrap();
    input.parse_terminated(Expr::parse)?
   } else {
    Punctuated::new()
   },
  })
 }
}

#[derive(Debug)]
struct MembersAndCasters {
 members: Vec<(Ident, Ident, usize)>,
 struct_members: Vec<proc_macro2::TokenStream>,
 row_casters: Vec<proc_macro2::TokenStream>,
}

impl MembersAndCasters {
 fn create(members: Vec<(Ident, Ident, usize)>) -> MembersAndCasters {
  let struct_members: Vec<_> = members.iter().map(|(name, ty, _i)| quote!(#name: #ty)).collect();
  let row_casters =
   members.iter().map(|(name, _ty, i)| quote!(#name: row.get(#i)?)).collect::<Vec<_>>();

  Self { members, struct_members, row_casters }
 }
}

fn extract_explicit_members(sql: &str) -> MembersAndCasters {
 let members: Vec<_> = onig::Regex::new(
  r"(?<!.*\s[Ff][Rr][Oo][Mm]\s.*)[a-zA-Z_][a-zA-Z_0-9]*_(String|f64|i64|i32|bool)",
 )
 .unwrap()
 .captures_iter(&sql.replace("\n", " ")) // Newlines break the lookbehind
 .enumerate()
 .map(|(i, cap)| {
  let col_name = cap.at(0).unwrap();
  let mut parts: Vec<_> = col_name.split("_").collect();
  let ty = parts.pop().unwrap();
  let name = parts.join("_");
  (format_ident!("{}", name), format_ident!("{}", ty), i)
 })
 .collect();

 MembersAndCasters::create(members)
}

fn extract_stmt_members(stmt: &Statement, span: &Span) -> MembersAndCasters {
 let members: Vec<_> = stmt
  .column_names()
  .iter()
  .enumerate()
  .map(|(i, col_name)| {
   let mut parts: Vec<_> = col_name.split("_").collect();

   if parts.len() < 2 {
    abort!(
     span,
     "SQL column name {:#?} must include a type annotation, e.g. {}_String or {}_i64.",
     col_name,
     col_name,
     col_name
    )
   }

   let ty = parts.pop().unwrap();

   match ty {
    "i64" | "String" => (),
    _ => abort!(span, "Invalid type annotation \"_{}\", try e.g. _String or _i64.", ty),
   }

   let name = parts.join("_");

   (format_ident!("{}", name), format_ident!("{}", ty), i)
  })
  .collect();

 // let struct_members: Vec<_> = members.iter().map(|(name, ty, _i)| quote!(#name: #ty)).collect();
 // let row_casters: Vec<_> =
 //  members.iter().map(|(name, _ty, i)| quote!(#name: row.get(#i).unwrap())).collect();

 MembersAndCasters::create(members)
}

enum ParseStatementType {
 Execute,
 Select,
 SelectCTE,
}
use ParseStatementType::{Execute, Select, SelectCTE};

#[derive(Debug)]
struct StatementInfo {
 parameter_count: usize,
 column_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct MigrationsToml {
 migrations_append_only: Option<Vec<String>>,
 target_schema_autogenerated: Option<String>,
}

fn migrations_to_tempdb(migrations: &Vec<String>) -> Connection {
 let tempdb = rusqlite::Connection::open_in_memory().unwrap();

 tempdb
  .execute_batch(
   "CREATE TABLE turbosql_migrations (rowid INTEGER PRIMARY KEY, migration TEXT NOT NULL);",
  )
  .unwrap();

 migrations.iter().for_each(|m| match tempdb.execute(m, params![]) {
  Ok(_) => (),
  Err(rusqlite::Error::ExecuteReturnedResults) => (), // pragmas
  Err(e) => abort_call_site!("Running migrations on temp db: {:?}", e),
 });

 tempdb
}

fn migrations_to_schema(migrations: &Vec<String>) -> String {
 migrations_to_tempdb(migrations)
  .prepare("SELECT sql FROM sqlite_master WHERE type='table' ORDER BY sql")
  .unwrap()
  .query_map(params![], |row| {
   let sql: String = row.get(0).unwrap();
   Ok(sql)
  })
  .unwrap()
  .map(|x| x.unwrap())
  .collect::<Vec<_>>()
  .join("\n")
}

fn read_migrations_toml() -> MigrationsToml {
 let lockfile = std::fs::File::create(std::env::temp_dir().join("migrations.toml.lock")).unwrap();
 fs2::FileExt::lock_exclusive(&lockfile).unwrap();

 let migrations_toml_path = std::env::current_dir().unwrap().join(MIGRATIONS_FILENAME);
 let migrations_toml_path_lossy = migrations_toml_path.to_string_lossy();

 match migrations_toml_path.exists() {
  true => {
   let toml_str = std::fs::read_to_string(&migrations_toml_path)
    .unwrap_or_else(|e| abort_call_site!("Unable to read {}: {:?}", migrations_toml_path_lossy, e));

   let toml_decoded: MigrationsToml = toml::from_str(&toml_str).unwrap_or_else(|e| {
    abort_call_site!("Unable to decode toml in {}: {:?}", migrations_toml_path_lossy, e)
   });

   toml_decoded
  }
  false => MigrationsToml { ..Default::default() },
 }
}

/// Returns some info extracted from the statement
/// Aborts macro if invalid
fn validate_sql(sql: &str) -> Result<StatementInfo, rusqlite::Error> {
 let tempdb = migrations_to_tempdb(&read_migrations_toml().migrations_append_only.unwrap());

 let stmt = tempdb.prepare(&sql)?;

 Ok(StatementInfo {
  parameter_count: stmt.parameter_count(),
  column_names: stmt.column_names().iter().map(|s| s.to_string()).collect(),
 })
}

fn validate_sql_or_abort(sql: &str) -> StatementInfo {
 validate_sql(sql).unwrap_or_else(|e| {
  abort_call_site!(r#"Error validating SQL statement: "{}". SQL: {:?}"#, e, sql)
 })
}

fn do_parse_tokens(
 input: ParseStream,
 statement_type: ParseStatementType,
) -> syn::Result<proc_macro2::TokenStream> {
 let span = input.span();

 let result_type = input.parse::<Type>().ok();
 let pred = input.parse::<LitStr>()?.value();

 // See if we have any explicitly declared columns
 let explicit_members = extract_explicit_members(&pred);

 let explicit_members =
  if explicit_members.members.is_empty() { None } else { Some(explicit_members) };

 // test sql statement for validity with just adding SELECT
 // TODO: use this same technique to check for CTE statements
 //       that start with WITH instead of SELECT before adding SELECT

 let stmt_info = if let Select = statement_type {
  match validate_sql(&format!("SELECT {}", pred)) {
   Ok(stmt_info) => Some(stmt_info),
   Err(_) => None,
  }
 } else {
  None
 };

 eprintln!("{}, {:?}", pred, stmt_info);

 let sql = match (&statement_type, &result_type, &stmt_info, &explicit_members) {
  (Select, Some(_result_type), Some(_stmt_info), None) => {
   // result type and stmt_info
   format!("SELECT {}", pred)
  }
  (Select, Some(result_type), None, None) => {
   // result type and no explicit_members
   let table_name = quote!(#result_type).to_string().to_lowercase();
   let tables = TABLES.lock().unwrap();
   let table = tables.get(&table_name).unwrap_or_else(|| {
    abort!(
     span,
     "Table {:?} not found. Does struct {} exist and have #[derive(Turbosql)]?",
     table_name,
     quote!(#result_type).to_string()
    )
   });

   let column_names_str =
    table.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");

   format!("SELECT {} FROM {} {}", column_names_str, table_name, pred)
  }
  (Select, _, _, _) => format!("SELECT {}", pred),
  (SelectCTE, _, _, _) => pred,
  (Execute, _, _, _) => pred,
 };

 let stmt_info = validate_sql_or_abort(&sql);

 // get query params and validate their count against what the statement is expecting

 let QueryParams { params } = input.parse()?;

 if params.len() != stmt_info.parameter_count {
  abort!(
   span,
   "Expected {} bound parameter{}, got {}: {:?}",
   stmt_info.parameter_count,
   if stmt_info.parameter_count == 1 { "" } else { "s" },
   params.len(),
   sql
  );
 }

 if !input.is_empty() {
  return Err(input.error("Expected parameters"));
 }

 // dispatch

 let (struct_members, row_casters) =
  match (&statement_type, &result_type, &stmt_info, explicit_members) {
   (Select, Some(_result_type), stmt_info, None) => {
    let members: Vec<_> = stmt_info
     .column_names
     .iter()
     .enumerate()
     .map(|(i, col_name)| (format_ident!("{}", col_name), format_ident!("None"), i))
     .collect();

    let m = MembersAndCasters::create(members);

    (m.struct_members, m.row_casters)
   }

   (Select, _, _, Some(m)) => (m.struct_members, m.row_casters),
   (SelectCTE, _, _, Some(m)) => (m.struct_members, m.row_casters),
   (Execute, _, _, _) => {
    let tokens = quote! {
    {
     (|| -> Result<_, _> {
      let db = ::turbosql::__TURBOSQL_DB.lock().unwrap();
      let mut stmt = db.prepare_cached(#sql)?;
      stmt.execute(::turbosql::params![#params])
     })()
    }
    };

    if !input.is_empty() {
     return Err(input.error("Expected parameters or ')'"));
    }

    return Ok(tokens);
   }
   _ => abort!(span, "Expected explicitly typed return values or a return type."),
  };

 let (result_type, struct_decl) = match result_type {
  Some(t) => (quote!(#t), None),
  // Some(t) => (quote!(#t), None, Some(quote!(, ..Default::default()))),
  None => {
   let tsr = format_ident!("TurbosqlResult");
   (
    quote!(#tsr),
    Some(quote! {
     #[derive(Debug, Clone, ::turbosql::Serialize)]
     struct #tsr { #(#struct_members),* }
    }),
   )
  }
 };

 let tokens = quote! {
  {
   #struct_decl
   (|| -> Result<Vec<#result_type>, ::turbosql::Error> {
    let db = ::turbosql::__TURBOSQL_DB.lock().unwrap();
    let mut stmt = db.prepare_cached(#sql)?;
    let result = stmt.query_map(::turbosql::params![#params], |row| {
     Ok(#result_type {
      #(#row_casters),*
      // #default
     })
    })?.collect::<Vec<_>>();

    let result = result.iter().flatten().cloned().collect::<Vec<_>>();

    Ok(result)
   })()
  }
 };

 if !input.is_empty() {
  return Err(input.error("Expected parameters or ')'"));
 }

 // eprintln!("{}", tokens);

 Ok(tokens)
}

impl Parse for SelectTokens {
 fn parse(input: ParseStream) -> syn::Result<Self> {
  Ok(SelectTokens { tokens: do_parse_tokens(input, Select)? })
 }
}

impl Parse for SelectCTETokens {
 fn parse(input: ParseStream) -> syn::Result<Self> {
  Ok(SelectCTETokens { tokens: do_parse_tokens(input, SelectCTE)? })
 }
}

impl Parse for ExecuteTokens {
 fn parse(input: ParseStream) -> syn::Result<Self> {
  Ok(ExecuteTokens { tokens: do_parse_tokens(input, Execute)? })
 }
}

// impl Parse for SelectTokens {
//  fn parse(input: ParseStream) -> syn::Result<Self> {
// let mut span = input.span();
// let test_db = TEST_DB.lock().unwrap();

// let (tokens, params, stmt, sql) = match input.parse::<Type>() {
//  Err(_) => {
// No table name, make magic anonymous type
// let pred: LitStr = input.parse()?;

// let sql = format!("SELECT {}", pred.value());

// let stmt = test_db.prepare(&sql).unwrap_or_else(|e| {
//  abort!(span, "Error verifying Turbosql-generated SELECT statement: {:?} {:?}", sql, e)
// });

// let members: Vec<_> = stmt
//  .column_names()
//  .iter()
//  .enumerate()
//  .map(|(i, col_name)| {
//   let mut parts: Vec<_> = col_name.split("_").collect();

//   if parts.len() < 2 {
//    abort!(
//     span,
//     "SQL column name {:#?} must include a type annotation, e.g. {}_String or {}_i64.",
//     col_name,
//     col_name,
//     col_name
//    )
//   }

//   let ty = parts.pop().unwrap();

//   match ty {
//    "i64" | "String" => (),
//    _ => abort!(span, "Invalid type annotation \"_{}\", try e.g. _String or _i64.", ty),
//   }

//   let name = parts.join("_");

//   (format_ident!("{}", name), format_ident!("{}", ty), i)
//  })
//  .collect();

// let struct_members: Vec<_> = members.iter().map(|(name, ty, _i)| quote!(#name: #ty)).collect();
// let row_casters: Vec<_> =
//  members.iter().map(|(name, _ty, i)| quote!(#name: row.get(#i).unwrap())).collect();

//     span = input.span();
//     let QueryParams { params } = input.parse()?;

//     let tokens = quote! {
//      {
//       #[derive(Debug)]
//       struct TurbosqlResult { #(#struct_members),* }
//       (|| -> Result<Vec<Result<TurbosqlResult, ::turbosql::Error>>, ::turbosql::Error> {
//        let db = ::turbosql::__TURBOSQL_DB.lock().unwrap();
//        let mut stmt = db.prepare_cached(#sql)?;
//        let result = Ok(stmt.query_map(::turbosql::params![#params], |row| {
//         Ok(TurbosqlResult {
//          #(#row_casters),*
//         }
//        )})?.collect::<Vec<_>>());
//        result
//       })()
//      }
//     };

//     (tokens, params, stmt, sql)
//    }
//    Ok(ty) => {
//     // Table
//     let table_name = quote!(#ty).to_string().to_lowercase();
//     let tables = TABLES.lock().unwrap();
//     let table = tables.get(&table_name).unwrap_or_else(|| {
//      abort!(
//       span,
//       "Table {:?} not found. Does struct {} exist and have #[derive(Turbosql)]?",
//       table_name,
//       quote!(#ty).to_string()
//      )
//     });

//     // Predicate

//     span = input.span();
//     let pred: LitStr = input.parse()?;

//     // Generate and validate SELECT statement

//     let column_names_str =
//      table.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");

//     let sql = format!("SELECT {} FROM {} {}", column_names_str, table_name, pred.value());

//     let stmt = test_db.prepare(&sql).unwrap_or_else(|e| {
//      abort!(span, "Error verifying Turbosql-generated SELECT statement: {:?} {:?}", sql, e)
//     });

//     span = input.span();
//     let QueryParams { params } = input.parse()?;

// let tokens = quote!(
//  #ty::__select_sql(#sql, ::turbosql::params![#params])
// );

//     (tokens, params, stmt, sql)
//    }
//   };

// if params.len() != stmt.parameter_count() {
//  abort!(
//   span,
//   "Expected {} bound parameter{}, got {}: {:?}",
//   stmt.parameter_count(),
//   if stmt.parameter_count() == 1 { "" } else { "s" },
//   params.len(),
//   sql
//  );
// }

// if !input.is_empty() {
//  return Err(input.error("Expected parameters"));
// }

// Ok(SelectTokens { tokens })
//  }
// }

/// Executes a SQL statement.
#[proc_macro]
#[proc_macro_error]
pub fn execute(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
 let ExecuteTokens { tokens } = parse_macro_input!(input);
 proc_macro::TokenStream::from(tokens)
}

/// Executes a SQL SELECT statement with automatic `SELECT` and `FROM` clauses.
#[proc_macro]
#[proc_macro_error]
pub fn select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
 let SelectTokens { tokens } = parse_macro_input!(input);
 proc_macro::TokenStream::from(tokens)
}

/// Executes a SQL "WITH ... SELECT" (CTE) statement
#[proc_macro]
#[proc_macro_error]
pub fn select_cte(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
 let SelectCTETokens { tokens } = parse_macro_input!(input);
 proc_macro::TokenStream::from(tokens)
}

/// Derive this on a `struct` to create a corresponding SQLite table and `insert`/`update`/`upsert` methods. (TODO: `Turbosql` trait?)
#[proc_macro_derive(Turbosql, attributes(turbosql))]
#[proc_macro_error]
pub fn turbosql_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
 // parse tokenstream and set up table struct

 let input = parse_macro_input!(input as DeriveInput);
 let table_span = input.span();
 let table_ident = input.ident;
 let table_name = table_ident.to_string().to_lowercase();

 let ltn = LAST_TABLE_NAME.lock().unwrap().clone();

 let mut last_table_name_ref = LAST_TABLE_NAME.lock().unwrap();
 *last_table_name_ref = format!("{}, {}", ltn, table_name);

 let fields = match input.data {
  Data::Struct(ref data) => match data.fields {
   Fields::Named(ref fields) => fields,
   Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  },
  Data::Enum(_) | Data::Union(_) => unimplemented!(),
 };

 let table = Table {
  ident: table_ident,
  span: table_span,
  name: table_name.clone(),
  columns: extract_columns(fields),
 };

 let minitable = MiniTable {
  name: table_name,
  columns: table
   .columns
   .iter()
   .map(|c| MiniColumn { name: c.name.clone(), sqltype: c.sqltype })
   .collect(),
 };

 TABLES.lock().unwrap().insert(minitable.name.clone(), minitable);

 // create trait functions

 let fn_create = create::create(&table);
 let fn_insert = insert::insert(&table);
 let fn_select = select::select(&table);

 // output tokenstream

 proc_macro::TokenStream::from(quote! {
  impl #table {
   #fn_create
   #fn_insert
   #fn_select
  }
 })
}

/// Convert syn::FieldsNamed to our Column type.
fn extract_columns(fields: &FieldsNamed) -> Vec<Column> {
 let columns = fields
  .named
  .iter()
  .filter_map(|f| {
   // Skip (skip) fields

   for attr in &f.attrs {
    let meta = attr.parse_meta().unwrap();
    match meta {
     Meta::List(list) if list.path.is_ident("turbosql") => {
      for value in list.nested.iter() {
       match value {
        NestedMeta::Meta(meta) => match meta {
         Meta::Path(p) if p.is_ident("skip") => {
          // TODO: For skipped fields, Handle derive(Default) requirement better
          // require Option and manifest None values
          return None;
         }
         _ => (),
        },
        _ => (),
       }
      }
     }
     _ => (),
    }
   }

   let ident = &f.ident;
   let name = ident.as_ref().unwrap().to_string();

   let ty = &f.ty;
   let ty_str = quote!(#ty).to_string();

   // TODO: have specific error messages or advice for other numeric types
   // specifically, sqlite cannot represent u64 integers, would be coerced to float.
   // https://sqlite.org/fileformat.html

   let sqltype = match (name.as_str(), ty_str.as_str()) {
    ("rowid", "Option < i64 >") => "INTEGER PRIMARY KEY",
    // (_, "i64") => "INTEGER NOT NULL",
    (_, "Option < i8 >") => "INTEGER",
    (_, "Option < u8 >") => "INTEGER",
    (_, "Option < i16 >") => "INTEGER",
    (_, "Option < u16 >") => "INTEGER",
    (_, "Option < i32 >") => "INTEGER",
    (_, "Option < u32 >") => "INTEGER",
    (_, "Option < i53 >") => "INTEGER",
    (_, "Option < i64 >") => "INTEGER",
    (_, "u64") => abort!(ty, SQLITE_64BIT_ERROR),
    (_, "Option < u64 >") => abort!(ty, SQLITE_64BIT_ERROR),
    // (_, "f64") => "REAL NOT NULL",
    (_, "Option < f64 >") => "REAL",
    // (_, "bool") => "BOOLEAN NOT NULL",
    (_, "Option < bool >") => "BOOLEAN",
    // (_, "String") => "TEXT NOT NULL",
    (_, "Option < String >") => "TEXT",
    // SELECT LENGTH(blob_column) ... will be null if blob is null
    // (_, "Blob") => "BLOB NOT NULL",
    (_, "Option < Blob >") => "BLOB",
    _ => abort!(ty, "turbosql doesn't support rust type: {}", ty_str),
   };

   Some(Column { ident: ident.clone().unwrap(), span: ty.span(), name, sqltype })
  })
  .collect::<Vec<_>>();

 // Make sure we have a rowid column, to keep a persistent rowid for blob access.
 // see https://www.sqlite.org/rowidtable.html :
 // "If the rowid is not aliased by INTEGER PRIMARY KEY then it is not persistent and might change."

 if !matches!(
  columns.iter().find(|c| c.name == "rowid"),
  Some(Column { sqltype: "INTEGER PRIMARY KEY", .. })
 ) {
  abort_call_site!("derive(Turbosql) structs must include a 'rowid: Option<i64>' field")
 };

 columns
}
