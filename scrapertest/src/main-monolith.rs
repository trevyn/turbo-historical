use anyhow::Result;
use reqwest::header;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use turbosql::{select, Turbosql};
use warp::http::Method;
use warp::Filter;

#[derive(Turbosql, Serialize, Debug, Default)]
struct ResultItem {
 rowid: Option<i64>,
 url: Option<String>,
 title: Option<String>,
 snippet: Option<String>,
}

fn make_new_db(path: &Path, json: &str) -> Result<meilisearch_wrapper::Database> {
 eprintln!("making new database!");

 // pub searchable_attributes: Option<Option<Vec<String>>>,
 // pub displayed_attributes: Option<Option<HashSet<String>>>,

 let settings = meilisearch_wrapper::Settings {
  searchable_attributes: Some(Some(vec!["title".to_owned(), "snippet".to_owned()])),
  displayed_attributes: Some(Some(HashSet::from_iter(vec![
   "rowid".to_owned(),
   "title".to_owned(),
   "snippet".to_owned(),
  ]))),
  ..Default::default()
 };

 //  let settings: meilisearch_wrapper::Settings = serde_json::from_str(
 //   r#"
 // {
 //  "searchableAttributes": ["title", "snippet"],
 //  "displayedAttributes": [
 //   "rowid",
 //   "title",
 //   "snippet"
 //  ]
 // }
 // "#,
 //  )
 //  .unwrap();

 let documents: serde_json::Value = serde_json::from_str(&json)?;
 let documents = documents.as_array().unwrap();

 meilisearch_wrapper::create_database(path, &settings, documents)
}

type Db = Arc<Mutex<meilisearch_wrapper::Database>>;

#[tokio::main]
async fn main() {
 let results = select!(ResultItem "ORDER BY rowid");
 eprintln!("{} rows", results.len());

 let path = Path::new("/Users/eden/Desktop/meili4/");

 // std::fs::create_dir(path).unwrap();
 // let serialized = serde_json::to_string(&results).unwrap();
 // let db = make_new_db(&path, &serialized).unwrap();

 let db = meilisearch_wrapper::open_database(&path).unwrap();
 let db = Arc::new(Mutex::new(db));

 eprintln!("starting warp");

 let api = warp::path!("hellofast" / String).and(with_db(db)).and_then(hellofast_handler);

 let api = api.or(
  warp::path!("hello" / String).and(warp::header::<String>("user-agent")).and_then(hello_handler),
 );

 let api = api.or(warp::path!("monolith" / String).and_then(monolith_handler));

 let cors = warp::cors()
  .allow_methods(&[Method::GET, Method::POST, Method::DELETE])
  .allow_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
  .allow_any_origin();

 let api = api.with(cors);
 let api = api.with(warp::log("todos"));

 warp::serve(api).run(([0, 0, 0, 0], 3030)).await;
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
 warp::any().map(move || db.clone())
}

async fn do_scrape(query: &str, agent: &str) -> Vec<ResultItem> {
 let req_url = format!("https://html.duckduckgo.com/html?q={}", query);

 let client = reqwest::Client::new();
 let req = client.get(&req_url).header(header::USER_AGENT, agent);
 let res = req.send().await.unwrap();
 let html = res.text().await.unwrap();
 let document = Html::parse_document(&html);

 let links_main = Selector::parse(r#".links_main"#).unwrap();
 let result_snippet = Selector::parse(r#".result__snippet"#).unwrap();
 let result_a = Selector::parse(r#".result__a"#).unwrap();

 let nodes = document.select(&links_main);

 nodes
  .map(|node| {
   let url = node.select(&result_a).next().unwrap().value().attr("href").unwrap();
   let title = node.select(&result_a).next().unwrap().inner_html();
   let snippet = node.select(&result_snippet).next().unwrap().inner_html();

   //   let url = Url::parse(format!("https:{}", url).as_str()).unwrap();
   //   let hash_query: HashMap<_, _> = url.query_pairs().into_owned().collect();
   //   let url = hash_query.get("uddg").unwrap();

   ResultItem { url: Some(url.to_owned()), title: Some(title), snippet: Some(snippet), rowid: None }
  })
  .collect::<Vec<_>>()
}

async fn monolith_handler(param: String) -> Result<impl warp::Reply, warp::Rejection> {
 let page = reqwest::get(&urldecode::decode(param)).await.unwrap().text().await.unwrap();

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

 println!("page is {} bytes", page.len());
 Ok(warp::reply::with_header(page, "content-type", "text/html"))
}

async fn hello_handler(param: String, agent: String) -> Result<impl warp::Reply, warp::Rejection> {
 let results = do_scrape(&param, &agent).await;
 ResultItem::insert_batch(&results);
 Ok(serde_json::to_string(&results).unwrap())
}

async fn hellofast_handler(param: String, db: Db) -> Result<impl warp::Reply, warp::Rejection> {
 let db = db.lock().await;

 let results = meilisearch_wrapper::query(&db, &urldecode::decode(param));

 let results = results
  .unwrap()
  .into_iter()
  .map(|rowid| select!(ResultItem "WHERE rowid = ?", rowid as i64))
  .collect::<Vec<_>>()
  .into_iter()
  .flatten()
  .collect::<Vec<_>>();

 // eprintln!("{:#?}", row[0].title.clone().unwrap());

 eprintln!("{:#?}", results);

 // Ok("{}")
 Ok(serde_json::to_string(&results).unwrap())
}
