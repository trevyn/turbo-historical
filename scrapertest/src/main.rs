use anyhow::Context;
use clap::Clap;
use juniper::{graphql_object, EmptySubscription, FieldResult, GraphQLObject};
use log::{debug, error, info, trace, warn};
use once_cell::sync::Lazy;
use reqwest::header;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::convert::Infallible;
use std::io::prelude::*;
use std::os::raw::{c_char, c_longlong, c_uchar};
use std::sync::Mutex;
use std::time::SystemTime;
use std::{
 ffi::{CStr, CString},
 pin::Pin,
};
use sysinfo::SystemExt;
use tokio::sync::{mpsc, oneshot};
use tokio::task::spawn_blocking;
use turbosql::{execute, select, Blob, Turbosql};
use url::Url;
use warp::http::{HeaderMap, Method};
use warp::Filter;

use bytes::Bytes;
use futures::stream::Stream;
use futures::task::Poll;
// use tokio::io::AsyncRead;
// use tokio::prelude::*;
// use tokio_util::codec;

#[allow(dead_code, non_snake_case)]
extern "C" {
 fn GoListJSON(path: *const c_char);
 fn GoSetConfig(path: *const c_char);
 fn GoFetchFiledata(path: *const c_char, startbytepos: c_longlong, endbytepos: c_longlong);
}

trait Ok<T> {
 fn ok(self) -> Result<T, anyhow::Error>;
}

impl<T> Ok<T> for Option<T> {
 fn ok(self) -> Result<T, anyhow::Error> {
  self.ok_or_else(|| anyhow::anyhow!("NoneError"))
 }
}

macro_rules! here {
 () => {
  concat!("at ", file!(), " line ", line!(), " column ", column!())
 };
}

#[derive(rust_embed::RustEmbed)]
#[folder = "../turbo_frontend/build"]
struct Asset;

// #[cfg(test)]
// mod tests {
//  use super::*;

// #[test]
// fn test_go_listjson() {
//  let start = Instant::now();

//  eprintln!("testing");

//  let config = CString::new("").unwrap(); // rclone.conf
//  unsafe {
//   GoSetConfig(config.as_ptr());
//  }

//  let path = "".to_owned();
//  let cstring = CString::new(path).unwrap();

//  unsafe {
//   GoListJSON(cstring.as_ptr());
//  }

// turbosql::execute(
//  "DELETE from file WHERE refresh_pending_verify = true AND origin_parent_path = ?",
//  params![path],
// )
// .unwrap();

// eprintln!("the file is {:#?}", files.iter().filter_map(|f| f.path.clone()).collect::<Vec<_>>());

// eprintln!("test_go_listjson complete in {}!", format!("{:.2?}", start.elapsed()).green().bold());
//  }
// }

/// Receive an array of File entries from Go and insert into turbosql
/// # Safety
/// `json` must be a valid pointer to valid C string until this function returns.
#[no_mangle]
extern "C" fn rust_insert_files_from_go(json: *const c_char) {
 let c_str = unsafe { CStr::from_ptr(json) };
 let string = c_str.to_str().unwrap().to_owned();

 let mut sender = RESPONSE_TX_CHANNEL.lock().unwrap().clone().unwrap();

 tokio::spawn(async move {
  sender.send(string).await.unwrap();
 });
}

#[derive(Turbosql, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
struct FileCache {
 #[serde(skip)]
 rowid: Option<i64>,
 cachekey: Option<String>,
 startbytepos: Option<i64>,
 endbytepos: Option<i64>,
 #[serde(skip)]
 bytes: Option<Blob>,
}

#[derive(Turbosql, Clone, Debug, Default)]
struct Rcloneconf {
 rowid: Option<i64>,
 conf: Option<String>,
}

#[derive(Turbosql, Clone, Debug, Default)]
struct FileKnowledge {
 rowid: Option<i64>,
 file_id: Option<i64>,
 kind: Option<String>,
 value: Option<String>,
}
/// Receive a Filecache entry from Go and insert into turbosql
/// buf is only valid until function return, must be copied
#[no_mangle]
extern "C" fn rust_insert_filecache_from_go(
 json: *const c_char,
 buf: *const c_uchar,
 len: c_longlong,
) {
 let c_str = unsafe { CStr::from_ptr(json) };
 let str = c_str.to_str().unwrap();

 log::info!("rust_insert_filecache_from_go: {:#?}", str);

 let mut fc: FileCache = serde_json::from_str(str).unwrap();

 log::info!("rust_insert_filecache_from_go fc: {:#?}", fc);

 let slice = unsafe { std::slice::from_raw_parts(buf, len as usize) };
 fc.bytes = Some(slice.to_vec());

 fc.insert().unwrap();
}

// let (listjson_tx, mut listjson_rx) = mpsc::channel::<(String)>(100);

static RESPONSE_TX_CHANNEL: Lazy<Mutex<Option<mpsc::Sender<String>>>> =
 Lazy::new(|| Mutex::new(None));

// static INSERT_FILES_CHANNEL: Lazy<Mutex<(Sender<String>, Receiver<String>)>> =
//  Lazy::new(|| Mutex::new(mpsc::channel::<String>(100)));

static LAST_SCRAPE: Lazy<Mutex<SystemTime>> = Lazy::new(|| Mutex::new(SystemTime::now()));

#[derive(Clap, Debug)]
struct Opts {
 #[clap(short, long)]
 cert_path: Option<String>,

 #[clap(short, long)]
 key_path: Option<String>,

 #[clap(short, long, default_value = "3020")]
 port: u16,

 /// GraphQL "Authentication: Bearer" password
 #[clap(long)]
 password: String,
}

#[derive(GraphQLObject, Turbosql, Clone, Debug, Default)]
struct ResultItem {
 #[graphql(skip)]
 rowid: Option<i64>,
 url: Option<String>,
 host: Option<String>,
 title: Option<String>,
 snippet: Option<String>,
 source_query: Option<String>,
 source_query_url: Option<String>,
 source_result_pos: Option<i32>,
 last_scraped: Option<f64>,
}

#[derive(GraphQLObject, Clone, Debug)]
struct SearchQueryResultItem {
 search_highlighted_url: String,
 title: String,
 snippet: String,
 url: String,
 host: String,
 bookmarked: bool,
 hostaffection: i32,
 rank: f64,
}

#[derive(GraphQLObject, Clone, Debug)]
struct BookmarkQueryResultItem {
 bookmark_timestamp: f64,
 title: String,
 snippet: String,
 url: String,
 host: String,
 bookmarked: bool,
 hostaffection: i32,
}

#[derive(GraphQLObject, Turbosql, Debug, Default, Clone)]
struct HostAffection {
 #[graphql(skip)]
 rowid: Option<i64>,
 host: Option<String>,
 affection: Option<i32>,
}
#[derive(GraphQLObject, Turbosql, Debug, Default, Clone)]
struct Bookmark {
 #[graphql(skip)]
 rowid: Option<i64>,
 url: Option<String>,
 timestamp: Option<f64>,
}

#[derive(GraphQLObject, Debug)]
struct ActivityMonitor {
 total_memory: i32,
 used_memory: i32,
 available_memory: i32,
 total_swap: i32,
 used_swap: i32,
}

mod mod_i53;
use mod_i53::i53;

#[derive(GraphQLObject, Turbosql, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
struct RcloneItem {
 #[graphql(skip)]
 #[serde(skip)]
 rowid: Option<i64>,
 #[serde(rename(deserialize = "ID"))]
 id: Option<String>,
 path: Option<String>,
 name: Option<String>,
 size: Option<i53>,
 mime_type: Option<String>,
 mod_time: Option<String>,
 is_dir: Option<bool>,
}

#[derive(GraphQLObject, Clone, Debug)]
struct RcloneItemQueryResultItem {
 path: String,
 name: String,
 size: i53,
 mime_type: String,
 mod_time: String,
 is_dir: bool,
 dir_size: Option<i53>,
}

struct Query;

#[graphql_object]
impl Query {
 async fn get_bookmarks() -> FieldResult<Vec<BookmarkQueryResultItem>> {
  Ok(select!(Vec<BookmarkQueryResultItem> r#"
   url,
   title,
   host,
   snippet,
   TRUE AS bookmarked,
   hostaffection,
   bookmark_timestamp
   FROM (
    SELECT
    bookmark.url AS url,
    resultitem.title AS title,
    resultitem.host AS host,
    resultitem.snippet AS snippet,
    hostaffection.affection AS hostaffection,
    bookmark.timestamp AS bookmark_timestamp
    FROM bookmark
    LEFT JOIN resultitem ON resultitem.url = bookmark.url
    LEFT JOIN hostaffection ON resultitem.host = hostaffection.host
    ORDER BY bookmark_timestamp DESC, resultitem.last_scraped DESC
   )
   GROUP BY url
   ORDER BY bookmark_timestamp DESC
  "#)?)
 }

 async fn search(query: String, force_scrape: bool) -> FieldResult<Vec<SearchQueryResultItem>> {
  if force_scrape {
   log::info!("scrape_search({:?})", query);
   scrape_search(query).await
  } else {
   log::info!("instant_search({:?})", query);
   instant_search(query).await
  }
 }

 async fn get_activity_monitor() -> FieldResult<ActivityMonitor> {
  // should be spawn_blocking
  let sys = sysinfo::System::new_all();
  Ok(ActivityMonitor {
   // these are all reported in kB, so i32 is fine. ;)
   total_memory: sys.get_total_memory() as i32,
   used_memory: sys.get_used_memory() as i32,
   available_memory: sys.get_available_memory() as i32,
   total_swap: sys.get_total_swap() as i32,
   used_swap: sys.get_used_swap() as i32,
  })
 }

 async fn get_commit_hash() -> FieldResult<String> {
  Ok(std::option_env!("GITHUB_SHA").unwrap_or("DEV").to_string())
 }

 async fn get_rclone_items(path: String) -> FieldResult<Vec<RcloneItemQueryResultItem>> {
  debug!("getRcloneItems(path:{:#?})", path);

  Ok(select!(Vec<RcloneItemQueryResultItem> r#"
   path,
   name,
   is_dir,
   mime_type,
   mod_time,
   size,
   (SELECT SUM(size) FROM rcloneitem rci2 WHERE rci1.is_dir AND rci2.path LIKE ? AND rci2.path LIKE rci1.path || "/%") AS dir_size
   FROM rcloneitem rci1
   WHERE path = ? || name"#, format!("{}%", path), path)?)
 }
}

fn convert_query_for_fts5(query: String) -> String {
 // some punctuation breaks it
 let mut query = query;
 query = query.replace(".", " ");
 query = query.replace("'", " ");
 query = query.replace(",", " ");
 query = query.replace("-", " ");

 // quotes must be balanced to work, so add a fake one at the end while we're typing
 if query.matches('"').count() % 2 == 1 {
  query.push('"');
 };

 // if there's no " on the end, use SQLite prefix search
 if let Some('"') = query.clone().pop() {
 } else {
  query.push('*');
 }

 query
}

async fn instant_search(query: String) -> FieldResult<Vec<SearchQueryResultItem>> {
 let match_query = convert_query_for_fts5(query);
 // let query = convert_query_for_fts5(query.clone()).split(" ").collect::<Vec<_>>().join(" OR ");

 log::info!("match_query = {:?}", match_query);

 Ok(select!(Vec<SearchQueryResultItem> r#"
  search_highlighted_url,
  title,
  snippet,
  sq.url AS url,
  sq.host AS host,
  bookmark.url IS NOT NULL AS bookmarked,
  IFNULL(hostaffection.affection, 0) AS hostaffection,
  MIN(sq.rank) AS rank
  
  FROM (
   SELECT
    highlight(resultitem2, 1, '<span class="search-highlight-url">', '</span>') AS search_highlighted_url,
    highlight(resultitem2, 2, '<span class="search-highlight">', '</span>') AS title,
    highlight(resultitem2, 3, '<span class="search-highlight">', '</span>') AS snippet,
    url,
    host,
    rank
    FROM resultitem2
    WHERE resultitem2 MATCH ?
    LIMIT -1 OFFSET 0  -- prevents "unable to use function highlight in the requested context"
  ) sq

  LEFT JOIN bookmark ON sq.url = bookmark.url
  LEFT JOIN hostaffection ON sq.host = hostaffection.host
  GROUP BY sq.url
  ORDER BY bookmarked DESC, hostaffection DESC, MIN(sq.rank)
  LIMIT 30
 "#, match_query)?)
}

async fn scrape_search(query: String) -> FieldResult<Vec<SearchQueryResultItem>> {
 // *LAST_SCRAPE.lock().unwrap() = SystemTime::now();
 let results =
  do_scrape(&query, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0")
   .await?;

 log::info!("scraped {} results", results.len());

 let scrapetime = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as f64;

 let results = results
  .iter()
  .enumerate()
  .map(|(i, r)| ResultItem {
   source_query: Some(query.clone()),
   source_query_url: Some(format!("https://html.duckduckgo.com/html?q={}", query)),
   source_result_pos: Some(i as i32),
   last_scraped: Some(scrapetime),
   ..r.clone()
  })
  .collect::<Vec<_>>();

 ResultItem::insert_batch(&results);

 for result in &results {
  execute!(
   "INSERT INTO resultitem2(url, title, snippet, host) VALUES (?, ?, ?, ?)",
   result.url,
   result.title,
   result.snippet,
   result.host
  )?;
 }

 // re-do search against database

 let match_query =
  convert_query_for_fts5(query.clone()).split(' ').collect::<Vec<_>>().join(" OR ");

 log::info!("match_query = {:?}", match_query);

 Ok(select!(Vec<SearchQueryResultItem> r#"
  search_highlighted_url,
  sq.title AS title,
  sq.snippet AS snippet,
  sq.url AS url,
  resultitem.host AS host,
  bookmark.url IS NOT NULL AS bookmarked,
  IFNULL(hostaffection.affection, 0) AS hostaffection,
  MIN(resultitem.source_result_pos) AS rank

  FROM (
   SELECT highlight(resultitem2, 1, '<span class="search-highlight-url">', '</span>') AS search_highlighted_url,
   highlight(resultitem2, 2, '<span class="search-highlight">', '</span>') AS title,
   highlight(resultitem2, 3, '<span class="search-highlight">', '</span>') AS snippet,
   url
   FROM resultitem2(?)
   WHERE resultitem2.url IN (SELECT DISTINCT url FROM resultitem WHERE source_query = ?)
   LIMIT -1 OFFSET 0  -- prevents "unable to use function highlight in the requested context"
  ) sq

  LEFT JOIN resultitem ON resultitem.url = sq.url AND resultitem.source_query = ?
  LEFT JOIN bookmark ON sq.url = bookmark.url
  LEFT JOIN hostaffection ON resultitem.host = hostaffection.host
  GROUP BY sq.url
  ORDER BY bookmarked DESC, hostaffection DESC, rank
  LIMIT 30
 "#,  match_query, query, query)?)
}

struct Mutations;

#[graphql_object]
impl Mutations {
 fn setHostAffection(host: String, affection: i32) -> FieldResult<String> {
  match affection {
   0 => {
    execute!("DELETE FROM hostaffection WHERE host = ?", host)?;
   }
   _ => {
    let host_affection: Vec<HostAffection> = select!(Vec<HostAffection> "WHERE host = ?", host)?;
    if host_affection.is_empty() {
     HostAffection { host: Some(host.clone()), affection: Some(affection), ..Default::default() }
      .insert()?;
    } else {
     execute!("UPDATE hostaffection SET affection = ? WHERE host = ?", affection, host)?;
    }
   }
  }
  Ok(host)
 }

 fn setBookmarked(url: String, bookmarked: bool) -> FieldResult<String> {
  match bookmarked {
   false => {
    execute!("DELETE FROM bookmark WHERE url = ?", url)?;
   }
   true => {
    let bookmark: Vec<Bookmark> = select!(Vec<Bookmark> "WHERE url = ?", url)?;
    if bookmark.is_empty() {
     Bookmark {
      url: Some(url.clone()),
      timestamp: Some(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as f64),
      ..Default::default()
     }
     .insert()?;
    }
   }
  }
  Ok(url)
 }
}

type Schema = juniper::RootNode<'static, Query, Mutations, EmptySubscription>;

#[derive(Debug)]
struct CustomReject;
impl warp::reject::Reject for CustomReject {}

fn anyhow_to_warp_rejection(err: anyhow::Error) -> warp::Rejection {
 error!("internal error occurred: {:#}", err);
 warp::reject::custom(CustomReject)
}

// fn anyhow_to_warp_rejection<E: Into<anyhow::Error>>(err: E) -> warp::Rejection {
//  error!("internal error occurred: {:#}", err.into());
//  warp::reject::custom(CustomReject)
// }

// fn pack(err: anyhow::Error) -> Problem {
//  let err = match err.downcast::<Problem>() {
//   Ok(problem) => return problem,

//   Err(err) => err,
//  };

//  // if let Some(err) = err.downcast_ref::<auth::AuthError>() {
//  //  match err {
//  //   auth::AuthError::InvalidCredentials => {
//  //    return Problem::new("Invalid credentials.")
//  //     .set_status(StatusCode::BAD_REQUEST)
//  //     .set_detail("The passed credentials were invalid.")
//  //   }

//  //   auth::AuthError::ArgonError => (),
//  //  }
//  // }

//  error!("internal error occurred: {:#}", err);
//  Problem::with_title_and_type_from_status(StatusCode::INTERNAL_SERVER_ERROR)
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
 // std::env::set_var("RUST_LOG", "trace");
 pretty_env_logger::init_timed();
 let warplog = warp::log("scrapertest");

 trace!("trace enabled");
 debug!("debug enabled");
 info!("info enabled");
 warn!("warn enabled");

 // info!("reading files!");
 // let contents = std::fs::read_to_string("/Users/eden/gcrypt.json")?;
 // let items: Vec<RcloneItem> = serde_json::from_str(&contents)?;
 // RcloneItem::insert_batch(&items);
 // info!("inserted!");
 // return Ok(());

 // info!("reading!");
 // let items: Vec<RcloneItem> = select!(Vec<RcloneItem>)?;
 // info!("read! {}", items.len());
 // let mut max_file_id = select!(i64 "MAX(rowid) FROM fileknowledge").unwrap_or(1);
 // info!("max_file_id! {}", max_file_id);

 // execute!("BEGIN TRANSACTION")?;

 // items.iter().map(|item| {
 //  if item.size.clone().ok()?.as_i64() > 0 {
 //   max_file_id += 1;
 //   execute!(
 //    r#"INSERT INTO fileknowledge (file_id, kind, value) VALUES (?, "name", ?), (?, "size", ?), (?, "localid", ?)"#,
 //    max_file_id, item.name,
 //    max_file_id, item.size,
 //    max_file_id, item.id
 //   )?;
 //  }
 //  Ok(())
 // }).collect::<anyhow::Result<Vec<()>>>()?;

 // execute!("COMMIT")?;

 // info!("committed!");
 // return Ok(());

 let opts = Opts::parse();
 let authorization = Box::leak(format!("Bearer {}", opts.password).into_boxed_str());

 // get rclone config

 let conf_path =
  directories_next::BaseDirs::new().unwrap().home_dir().join(".config/rclone/rclone.conf");

 let config = std::fs::read_to_string(&conf_path).unwrap_or_default();
 // .unwrap_or_else(|e| panic!("Something went wrong reading {:#?}: {}", conf_path, e));

 let config = CString::new(config).unwrap();
 // let config = CString::new(select!(Rcloneconf)?.conf.unwrap()).unwrap();
 // let config = CString::new(include_str!("rclone.conf")).unwrap();

 unsafe {
  GoSetConfig(config.as_ptr());
 }

 let (listjson_tx, mut listjson_rx) = mpsc::channel::<(String, oneshot::Sender<String>)>(1);
 let (response_tx, mut response_rx) = mpsc::channel::<String>(1);

 *RESPONSE_TX_CHANNEL.lock().unwrap() = Some(response_tx);

 tokio::spawn(async move {
  while let Some((path, response)) = listjson_rx.recv().await {
   let cstring = CString::new(path).unwrap();

   spawn_blocking(move || unsafe {
    GoListJSON(cstring.as_ptr());
   })
   .await
   .unwrap();

   let string = response_rx.recv().await.unwrap();

   response.send(string).ok();
  }
 });

 *LAST_SCRAPE.lock().unwrap() = SystemTime::now();

 let cors = warp::cors()
  .allow_methods(&[Method::GET, Method::POST])
  .allow_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
  .allow_any_origin();

 let filedl = warp::path("filedl").and(warp::header::headers_cloned()).and(warp::path::full());

 let api = warp::path("graphql")
  .and(warp::header::exact("authorization", authorization))
  .and(juniper_warp::make_graphql_filter(
   Schema::new(Query, Mutations, EmptySubscription::new()),
   warp::any().map(move || ()).boxed(),
  ))
  //
  .or(warp::get().and(warp::path("graphiql")).and(juniper_warp::graphiql_filter("/graphql", None)))
  .or(
   warp::get().and(warp::path("playground")).and(juniper_warp::playground_filter("/graphql", None)),
  )
  // warp::path("listjson")
  // .and(warp::filters::path::full())
  // .and(with_sender(listjson_tx))
  // .and_then(listjson_handler)
  //
  .or(filedl.and(warp::get()).and_then(filedl_get_handler))
  .or(filedl.and(warp::head()).and_then(filedl_head_handler))
  //
  // .or(warp::path!("monolith" / String).and_then(monolith_handler))
  //
  .or(warp::path("static").and(warp::path::full()).map(|path: warp::path::FullPath| {
   match (|| -> anyhow::Result<_> {
    Ok(warp::reply::with_header(
     std::str::from_utf8(
      Asset::get(path.as_str().trim_start_matches('/')).context(here!())?.as_ref(),
     )
     .unwrap()
     .to_string(),
     "content-type",
     "text/css",
    ))
   })() {
    Ok(body) => body,
    Err(_e) => panic!("panic!"), //warp::reply::html("error!".to_string()),
   }
  }))
  //
  .or(warp::path("favicon.ico").map(|| {
   Ok(
    warp::http::Response::builder()
     .header("content-type", "image/x-icon")
     .body(Asset::get("favicon.ico").unwrap()),
   )
  }))
  //
  .or(warp::any().map(|| {
   Ok(warp::reply::html(
    std::str::from_utf8(Asset::get("index.html").unwrap().as_ref()).unwrap().to_string(),
   ))
  }))
  //
  .with(cors)
  //
  .with(warplog);

 match (opts.key_path, opts.cert_path) {
  (Some(key_path), Some(cert_path)) => {
   log::info!("Serving HTTPS on port {}", opts.port);
   warp::serve(api)
    .tls()
    .cert_path(cert_path)
    .key_path(key_path)
    .run(([0, 0, 0, 0], opts.port))
    .await;
  }
  (None, None) => {
   log::info!("Serving (unsecured) HTTP on port {}", opts.port);
   warp::serve(api).run(([0, 0, 0, 0], opts.port)).await;
  }
  _ => panic!("Both key-path and cert-path must be specified for HTTPS."),
 }

 Ok(())
}

fn with_sender(
 sender: mpsc::Sender<(String, oneshot::Sender<String>)>,
) -> impl Filter<Extract = (mpsc::Sender<(String, oneshot::Sender<String>)>,), Error = Infallible> + Clone
{
 warp::any().map(move || sender.clone())
}

async fn do_scrape(query: &str, agent: &str) -> FieldResult<Vec<ResultItem>> {
 let req_url = format!("https://html.duckduckgo.com/html?q={}", query);

 let client = reqwest::Client::new();
 let req = client.get(&req_url).header(header::USER_AGENT, agent);
 let res = req.send().await?;
 let html = res.text().await?;
 let document = Html::parse_document(&html);

 let links_main = Selector::parse(r#".links_main"#).unwrap();
 let result_snippet = Selector::parse(r#".result__snippet"#).unwrap();
 let result_a = Selector::parse(r#".result__a"#).unwrap();

 let nodes = document.select(&links_main);

 let unbold_re = regex::Regex::new(r"<b>|</b>")?;

 Ok(
  nodes
   .filter_map(|node| {
    let url = node.select(&result_a).next()?.value().attr("href")?;

    if url.starts_with("https://duckduckgo.com/y.js?ad_provider=") {
     return None;
    }

    let title = node.select(&result_a).next()?.inner_html();
    let snippet = node.select(&result_snippet).next()?.inner_html();

    let parsed_url = Url::parse(url).unwrap();
    let host = parsed_url.host_str()?;

    //   let url = Url::parse(format!("https:{}", url).as_str())?;
    //   let hash_query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    //   let url = hash_query.get("uddg")?;

    Some(ResultItem {
     url: Some(url.to_owned()),
     title: Some((*unbold_re.replace_all(&title, "")).to_owned()),
     snippet: Some((*unbold_re.replace_all(&snippet, "")).to_owned()),
     host: Some(host.to_owned()),
     ..Default::default()
    })
   })
   .collect::<Vec<_>>(),
 )
}

async fn filedl_head_handler(
 headers: HeaderMap,
 fullpath: warp::path::FullPath,
) -> Result<impl warp::Reply, warp::Rejection> {
 log::info!("filedl_HEAD_handler {:#?} {:#?}", headers, fullpath);

 // let path = param.as_str().trim_start_matches("/files");

 // let path = CString::new(path).unwrap();

 // spawn_blocking(move || unsafe {
 //  GoFetchFiledata(path.as_ptr(), 0, 1500000);
 // })
 // .await
 // .unwrap();

 // pull data from

 Ok("hi")
}

struct ByteStream<R>(R);

impl<R: Read + Unpin> Stream for ByteStream<R> {
 // The same as our future above:
 type Item = tokio::io::Result<Bytes>;
 // type Error = std::io::Error;

 fn poll_next(
  mut self: Pin<&mut Self>,
  _cx: &mut futures::task::Context,
 ) -> Poll<Option<Self::Item>> {
  // let mut buf = [0u8; 1];
  let mut buf = vec![0_u8; 1024 * 1024];
  let n = self.0.read(&mut buf)?;
  // eprintln!("n: {:?}", n);
  buf.truncate(n);
  Poll::Ready(Some(Ok(Bytes::from(buf))))

  // let mut buf2 = vec![0_u8; 0];
  // let mut buf = BytesMut::with_capacity(1024 * 1024);

  // info!("bytesmut: {}", buf.len());
  // loop {
  //  match Pin::new(&mut self.0).poll_read(cx, &mut buf) {
  //   Poll::Ready(Ok(n)) => {
  //    // By convention, if an AsyncRead says that it read 0 bytes,
  //    // we should assume that it has got to the end, so we signal that
  //    // the Stream is done in this case by returning None:
  //    if n == 0 {
  //     info!("bytes none");
  //     break;
  //    } else {
  //     // info!("bytes inner: {:?}", n);
  //     buf.truncate(n);

  //     buf2.extend(buf.iter());
  //    }
  //   }
  //   Poll::Pending => {
  //    if buf2.len() == 0 {
  //     return Poll::Pending;
  //    }
  //    break;
  //   }
  //   // Err(e) => Err(e),
  //   Poll::Ready(Err(e)) => panic!("err {:#?}", e),
  //  }

  //  if buf2.len() > 1024 * 1024 {
  //   break;
  //  }
  // }
  // // info!("bytes outer---------------> : {:?}", buf2.len());

  // Poll::Ready(None)
  // Poll::Ready(Some(Ok(Bytes::from(buf2))))
 }
 // poll is very similar to our Future implementation, except that
 // it returns an `Option<u8>` instead of a `u8`. This is so that the
 // Stream can signal that it's finished by returning `None`:
 // fn poll_next(
 //  &mut self,
 //  _: &mut futures::task::Context,
 // ) -> futures::task::Poll<Option<Self::Item>> {
 //  let mut buf = [0; 1];
 //  match self.0.poll_read(&mut buf) {
 //   Ok(Async::Ready(n)) => {
 //    // By convention, if an AsyncRead says that it read 0 bytes,
 //    // we should assume that it has got to the end, so we signal that
 //    // the Stream is done in this case by returning None:
 //    if n == 0 {
 //     Ok(Async::Ready(None))
 //    } else {
 //     Ok(Async::Ready(Some(buf[0])))
 //    }
 //   }
 //   Ok(Async::NotReady) => Ok(Async::NotReady),
 //   Err(e) => Err(e),
 //  }
 // }
}
// let byte_stream1 = ByteStream(io::stdin());

async fn filedl_get_handler(
 headers: HeaderMap,
 fullpath: warp::path::FullPath,
) -> Result<impl warp::Reply, warp::Rejection> {
 match async {
  info!("filedl_get_handler {:#?} {:#?}", headers, fullpath);

  let path = fullpath.as_str().trim_start_matches("/filedl/");
  let path = urlencoding::decode(path)?;

  let rcloneitem = select!(RcloneItem "WHERE path = ?", &path).context(here!())?;
  let size = rcloneitem.size.unwrap().as_i64();
  let endbytepos = size - 1;

  // let cached_ranges =
  //  select!(Vec<_> "startbytepos, endbytepos FROM filecache WHERE cachekey = ?", &path);

  let filecache =
   match select!(Option<FileCache> "WHERE cachekey = ? AND startbytepos = ? AND endbytepos = ?",
  &path, 0, endbytepos)
   .context(here!())?
   {
    Some(fc) => fc,
    None => {
     let path_cstr = CString::new(path.clone())?;
     info!("starting fetch, {} bytes", size);
     spawn_blocking(move || unsafe {
      GoFetchFiledata(path_cstr.as_ptr(), 0, endbytepos);
     })
     .await?;
     info!("file fetched");
     select!(FileCache "WHERE cachekey = ? AND startbytepos = ? AND endbytepos = ?",
    path, 0, endbytepos)
     .context(here!())?
    }
   };

  info!("{:#?}", rcloneitem.mime_type);

  Ok(
   warp::http::Response::builder()
    // .header("content-type", "video/webm")
    .header("content-type", rcloneitem.mime_type.context(here!())?)
    .header("content-length", size)
    .header("accept-ranges", "bytes")
    // .body(warp::hyper::Body::wrap_stream(ByteStream(""))),
    .body(filecache.bytes.context(here!())?),
  )
 }
 .await
 {
  Ok(response) => Ok(response),
  Err(e) => Err(anyhow_to_warp_rejection(e)),
 }
}

// let f = tokio::fs::File::open("file.mkv").await.unwrap();

// https://stackoverflow.com/questions/59318460/what-is-the-best-way-to-convert-an-asyncread-to-a-trystream-of-bytes
// fn into_bytes_stream<R>(r: R) -> impl Stream<Item = tokio::io::Result<Bytes>>
// where
//  R: AsyncRead,
// {
//  codec::FramedRead::new(r, codec::BytesCodec::new()).map_ok(|bytes| {
//   info!("bytes: {:?}", bytes.len());
//   bytes.freeze()
//  })
// }

// let f2 = std::fs::File::open("file.mkv").unwrap();

// let metadata = std::fs::metadata("file.mkv").unwrap();

// let mut buffer = Vec::new();
// f2.read_to_end(&mut buffer).unwrap();

// Ok(
//  warp::http::Response::builder()
//   .header("content-type", "applicationvideo/webm")
//   .header("content-length", "937561057")
//   .header("accept-ranges", "bytes")
//   .body(buffer),
// )

// let path = param.as_str().trim_start_matches("/files");

// let path = CString::new(path).unwrap();

// spawn_blocking(move || unsafe {
//  GoFetchFiledata(path.as_ptr(), 0, 1500000);
// })
// .await
// .unwrap();

// pull data from
// Ok(warp::reply::with_header(buffer, "content-type", "video/webm"))

// https://stackoverflow.com/questions/59318460/what-is-the-best-way-to-convert-an-asyncread-to-a-trystream-of-bytes

// Ok(buffer)

// async fn monolith_handler(param: String) -> Result<impl warp::Reply, warp::Rejection> {
//  let page = reqwest::get(&urldecode::decode(param.clone())).await.unwrap().text().await.unwrap();

//  log::info!("page is {} bytes", page.len());
// fs::write("/Users/eden/Desktop/pre.html", &page).unwrap();

// let result = ammonia::clean(&page);

// let tags = ["link", "svg", "g", "rect", "polygon", "style"].iter();
// let tag_attributes = hashmap![
//   "link" => hashset!["href", "rel", "type", "as", "media"],
//   "img" => hashset!["src"],
//   "svg" => hashset!["width", "height", "viewBox", "version", "xmlns", "xmlns:xlink"],
//   "g" => hashset!["stroke", "stroke-width", "fill", "fill-rule"],
//   "rect" => hashset!["fill"],
//   "div" => hashset!["id"]
// ];

// let result = ammonia::Builder::default()
//  .add_tags(hashset!["link", "svg", "g", "rect", "polygon", "style", "wix-image", "path"])
//  .clean_content_tags(hashset!["script"])
//  .add_generic_attributes(hashset!["class", "style", "id"])
//  .add_tag_attributes("link", &["href", "rel", "type", "as", "media"])
//  .add_tag_attributes("svg", &["width", "height", "viewBox", "version", "xmlns", "xmlns:xlink"])
//  .add_tag_attributes("g", &["stroke", "stroke-width", "fill", "fill-rule"])
//  .add_tag_attributes("rect", &["fill", "x", "y", "width", "height", "rx"])
//  .add_tag_attributes("path", &["d"])
//  .url_relative(ammonia::UrlRelative::RewriteWithBase(
//   ammonia::Url::parse(&urldecode::decode(param)).unwrap(),
//  ))
//  .clean(&page)
//  .to_string();

// let dom = html5ever::parse_document(html5ever::rcdom::RcDom::default(), Default::default())
//  .from_utf8()
//  .read_from(&mut page.as_bytes())
//  .unwrap();

// println!("{:#?}", dom);

// let mut buf: Vec<u8> = Vec::new();
// html5ever::serialize::serialize(
//  &mut buf,
//  &dom.document,
//  html5ever::serialize::SerializeOpts::default(),
// )
// .expect("unable to serialize DOM into buffer");

// let result = String::from_utf8(buf).unwrap();

// html5ever::serialize
// let page = tokio::task::spawn_blocking(|| {
//  monolith_wrapper::get_page(monolith_wrapper::Options {
//   target: urldecode::decode(param),
//   no_css: false,
//   no_fonts: true,
//   no_frames: true,
//   no_images: false,
//   no_js: true,
//   insecure: false,
//   isolate: false,
//   output: "google.html".to_owned(),
//   silent: false,
//   timeout: 60,
//   user_agent: "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0"
//    .to_owned(),
//   no_metadata: true,
//  })
//  .unwrap()
// })
// .await
// .unwrap();
// fs::write("/Users/eden/Desktop/post.html", &result).unwrap();

// println!("recreated page is {} bytes", result.len());
//  Ok(warp::reply::with_header(page, "content-type", "text/html"))
// }

// async fn listjson_handler(
//  path: warp::filters::path::FullPath,
//  listjson_tx: mpsc::Sender<(String, oneshot::Sender<String>)>,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//  let path = path.as_str().trim_start_matches("/listjson/");
//  log::info!("path pre {}", path);

//  let path = urldecode::decode(path.to_owned());
//  log::info!("fetching {}", path);

//  let (resp_tx, resp_rx) = oneshot::channel();

//  listjson_tx.clone().send((path, resp_tx)).await.unwrap();

//  Ok(resp_rx.await.unwrap())
// }
