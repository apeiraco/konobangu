//! Fixed, single-call facade recipe. Policy features are activated in Cargo.
use image::DynamicImage;

use crate::{errors::RecorderResult, media::invalid_options};
pub(super) fn encode(image: DynamicImage) -> RecorderResult<Vec<u8>> {
  if image.color().has_alpha() {
    return Err(invalid_options(
      "JPXL preset does not support RGBA or lossless transparency; use a white cover or WebP",
    ));
  }
  let rgb = image.into_rgb8();
  jpxl::Encoder::new()
    .with_quality(77.0)
    .and_then(|encoder| encoder.with_effort(jpxl::Effort::Balanced).with_threads(1))
    .and_then(|encoder| encoder.encode_rgb8(rgb.width(), rgb.height(), rgb.as_raw()))
    .map_err(|_| invalid_options("JPXL Balanced 77 encoding failed"))
}
