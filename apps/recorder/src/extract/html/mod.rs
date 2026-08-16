pub mod styles;

use html_escape::decode_html_entities;
use itertools::Itertools;
use scraper::ElementRef;
pub use styles::{extract_background_image_src_from_style_attr, extract_style_from_attr};

pub fn extract_inner_text_from_element_ref(el: ElementRef<'_>) -> String {
  let raw_text = el.text().collect_vec().join(",");
  decode_html_entities(&raw_text).trim().to_string()
}
