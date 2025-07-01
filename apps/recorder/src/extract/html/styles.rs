use lightningcss::{
    declaration::DeclarationBlock, properties::Property, values::image::Image as CSSImage,
};
use url::Url;

use crate::extract::media::extract_image_src_from_str;

pub fn extract_style_from_attr(style_attr: &'_ str) -> Option<DeclarationBlock<'_>> {
    let result = DeclarationBlock::parse_string(style_attr, Default::default()).ok()?;
    Some(result)
}

pub fn extract_background_image_src_from_style_attr(
    style_attr: &str,
    base_url: &Url,
) -> Option<Url> {
    extract_style_from_attr(style_attr).and_then(|style| {
        style.iter().find_map(|(prop, _)| {
            match prop {
                Property::BackgroundImage(images) => {
                    for img in images {
                        if let CSSImage::Url(path) = img
                            && let Some(url) = extract_image_src_from_str(path.url.trim(), base_url)
                        {
                            return Some(url);
                        }
                    }
                }
                Property::Background(backgrounds) => {
                    for bg in backgrounds {
                        if let CSSImage::Url(path) = &bg.image
                            && let Some(url) = extract_image_src_from_str(path.url.trim(), base_url)
                        {
                            return Some(url);
                        }
                    }
                }
                _ => {}
            }
            None
        })
    })
}
