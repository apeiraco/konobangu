use url::Url;

pub fn extract_image_src_from_str(image_src: &str, base_url: &Url) -> Option<Url> {
  let mut image_url = base_url.join(image_src).ok()?;
  image_url.set_query(None);
  Some(image_url)
}
