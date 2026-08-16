//! Public offline release verification through the production media service.
use std::{ffi::OsString, io::Cursor, path::PathBuf};

use bytes::Bytes;
use clap::Parser;

use crate::{
  errors::RecorderResult,
  media::{MediaConfig, MediaService, derivative::hash, invalid_options},
};
#[derive(Parser)]
#[command(
  name = "recorder-cli media-smoke",
  about = "Encode a cover with the compiled production codecs, without starting the server"
)]
struct Arguments {
  #[arg(long)]
  input: Option<PathBuf>,
  #[arg(long)]
  output: PathBuf,
}
pub async fn run(arguments: impl IntoIterator<Item = OsString>) -> RecorderResult<()> {
  let arguments = Arguments::parse_from(arguments);
  let config = MediaConfig::default();
  let input: Bytes = if let Some(input) = arguments.input {
    if std::fs::metadata(&input)?.len() > config.max_input_bytes {
      return Err(invalid_options("Image input byte budget exceeded"));
    }
    std::fs::read(input)?.into()
  } else {
    let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(32, 24, |x, y| {
      image::Rgba([(x * 8) as u8, (y * 10) as u8, 128, ((x + y) * 4).min(255) as u8])
    }));
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png)?;
    out.into_inner().into()
  };
  let service = MediaService::from_config(config).await?;
  std::fs::create_dir_all(&arguments.output)?;
  std::fs::write(arguments.output.join("source.png"), &input)?;
  let normalized = super::normalize_cover(super::decode(&input, &service.config)?, &service.config)?;
  normalized.save(arguments.output.join("normalized.png"))?;
  let webp = service.optimize_cover_to_webp(input.clone(), None).await?;
  std::fs::write(arguments.output.join("encoded.webp"), &webp)?;
  let receipt = serde_json::json!({"source_sha256":hash(&input),"webp_sha256":hash(&webp),"webp_bytes":webp.len(),"jxl":cfg!(feature="jxl"),"width":normalized.width(),"height":normalized.height()});
  #[cfg(feature = "jxl")]
  let receipt = {
    let mut receipt = receipt;
    let jxl = service.optimize_cover_to_jxl(input, None).await?;
    std::fs::write(arguments.output.join("encoded.jxl"), &jxl)?;
    receipt["jxl_sha256"] = hash(&jxl).into();
    receipt["jxl_bytes"] = jxl.len().into();
    receipt
  };
  service.shutdown().await?;
  std::fs::write(arguments.output.join("receipt.json"), serde_json::to_vec_pretty(&receipt)?)?;
  println!("{receipt}");
  Ok(())
}
