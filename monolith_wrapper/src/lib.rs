use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::collections::HashMap;
use std::time::Duration;

use monolith::html::{
 add_base_tag, add_favicon, has_base_tag, has_favicon, html_to_dom, metadata_tag,
 stringify_document, walk_and_embed_assets,
};
use monolith::url::{
 data_to_data_url, data_url_to_data, is_data_url, is_file_url, is_http_url, resolve_url,
};
use monolith::utils::retrieve_asset;

pub use monolith::opts::Options;

use anyhow::{bail, Context};

// #[macro_export]
macro_rules! str {
 () => {
  String::new()
 };
 ($val: expr) => {
  ToString::to_string(&$val)
 };
}

// #[macro_export]
// macro_rules! empty_image {
//     () => {
// "data:image/png;base64,\
// iVBORw0KGgoAAAANSUhEUgAAAA0AAAANCAQAAADY4iz3AAAAEUlEQVR42mNkwAkYR6UolgIACvgADsuK6xYAAAAASUVORK5CYII="
//     };
// }

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn it_works() {
  let result = get_page(Options {
   target: "https://stackoverflow.com/questions/10418644/".to_owned(),
   no_css: false,
   no_fonts: false,
   no_frames: true,
   no_images: true,
   no_js: true,
   insecure: false,
   isolate: true,
   output: "google.html".to_owned(),
   silent: true,
   timeout: 60,
   user_agent: "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0"
    .to_owned(),
   no_metadata: true,
  });

  let size = result.unwrap().len();

  println!("monolithed page is {} bytes", size);

  assert!(size > 100_000);
 }
}

pub fn get_page(options: Options) -> anyhow::Result<String> {
 let original_target: &str = &options.target;
 let target_url: &str;
 let base_url;
 let mut dom;

 // Pre-process the input
 let target: String = str!(original_target.clone()).replace("\\", "/");

 // Determine exact target URL
 if target.clone().len() == 0 {
  bail!("No target specified");
 } else if is_http_url(target.clone()) || is_data_url(target.clone()) {
  target_url = target.as_str();
 } else {
  bail!("Not an HTTP/HTTPS or Data URL");
 }

 // Initialize client
 let mut cache = HashMap::new();
 let mut header_map = HeaderMap::new();
 header_map.insert(
  USER_AGENT,
  HeaderValue::from_str(&options.user_agent).context("Invalid User-Agent header specified")?,
 );
 let timeout: u64 = if options.timeout > 0 { options.timeout } else { std::u64::MAX / 4 };
 let client = Client::builder()
  .timeout(Duration::from_secs(timeout))
  .danger_accept_invalid_certs(options.insecure)
  .default_headers(header_map)
  .build()
  .context("Failed to initialize HTTP client")?;

 // Retrieve target document
 if is_file_url(target_url) || is_http_url(target_url) {
  match retrieve_asset(&mut cache, &client, target_url, target_url, options.silent, 0) {
   Ok((data, final_url, _media_type)) => {
    base_url = final_url;
    dom = html_to_dom(&String::from_utf8_lossy(&data));
   }
   Err(_) => {
    bail!("Could not retrieve target document");
   }
  }
 } else if is_data_url(target_url) {
  let (media_type, data): (String, Vec<u8>) = data_url_to_data(target_url);
  if !media_type.eq_ignore_ascii_case("text/html") {
   bail!("Unsupported data URL media type");
  }
  base_url = str!(target_url);
  dom = html_to_dom(&String::from_utf8_lossy(&data));
 } else {
  bail!("Unknown URL type");
 }

 // Embed remote assets
 walk_and_embed_assets(&mut cache, &client, &base_url, &dom.document, &options, 0);

 // Take care of BASE tag
 if is_http_url(base_url.clone()) && !has_base_tag(&dom.document) {
  dom = add_base_tag(&dom.document, base_url.clone());
 }

 // Request and embed /favicon.ico (unless it's already linked in the document)
 if !options.no_images && is_http_url(target_url) && !has_favicon(&dom.document) {
  let favicon_ico_url: String = resolve_url(&base_url, "/favicon.ico")?;

  match retrieve_asset(&mut cache, &client, &base_url, &favicon_ico_url, options.silent, 0) {
   Ok((data, final_url, media_type)) => {
    let favicon_data_url: String = data_to_data_url(&media_type, &data, &final_url);
    dom = add_favicon(&dom.document, favicon_data_url);
   }
   Err(_) => {
    // Failed to retrieve favicon.ico
   }
  }
 }

 // Serialize DOM tree
 let mut result: String = stringify_document(&dom.document, &options);

 // Add metadata tag
 if !options.no_metadata {
  let metadata_comment: String = metadata_tag(&base_url);
  result.insert_str(0, &metadata_comment);
  if metadata_comment.len() > 0 {
   result.insert_str(metadata_comment.len(), "\n");
  }
 }

 Ok(result)
}
