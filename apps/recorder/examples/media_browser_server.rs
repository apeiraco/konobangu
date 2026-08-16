//! Loopback browser verification through the production static controller.
use std::{path::PathBuf, sync::Arc};

use axum::{Router, extract::Query, response::Html, routing::get};
use recorder::{
  app::AppContextTrait,
  media::{
    AutoOptimizeImageFormat as F, MediaConfig, MediaService,
    derivative::{self, DerivativeEntry, DerivativeManifest},
  },
  storage::StorageService,
  test_utils::app::TestingAppContext,
  web::controller::ControllerTrait,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let directory = PathBuf::from(std::env::args().nth(1).expect("evidence directory"));
  std::fs::create_dir_all(&directory)?;
  let storage = StorageService {
    data_dir: directory.to_string_lossy().into(),
    operator: StorageService::get_operator(directory.to_str().unwrap())?,
  };
  let media = MediaService::from_config(MediaConfig::default()).await?;
  let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(256, 256, |x, y| image::Rgba([255, y as u8, 0, x as u8])));
  let mut input = std::io::Cursor::new(Vec::new());
  image.write_to(&mut input, image::ImageFormat::Png)?;
  let source = "public/browser-cover.png";
  storage.write(source, input.get_ref().clone().into()).await?;
  let mut entries = Vec::new();
  for plan in media.derivative_plans() {
    let encoded = match plan.format {
      F::Jxl => media.optimize_cover_to_jxl(input.get_ref().clone().into(), None).await?,
      F::Webp => media.optimize_cover_to_webp(input.get_ref().clone().into(), None).await?,
      _ => unreachable!(),
    };
    let sha256 = derivative::hash(&encoded);
    let path = derivative::derivative_path(source, &plan.profile, &derivative::hash(input.get_ref()), &sha256, plan.format);
    storage.write(&path, encoded.clone()).await?;
    entries.push(DerivativeEntry {
      profile: plan.profile,
      format: plan.format,
      path,
      length: encoded.len() as u64,
      sha256,
    });
  }
  let manifest = DerivativeManifest {
    version: derivative::VERSION,
    source_fingerprint: derivative::fingerprint(&storage.stat(source).await?).unwrap(),
    source_sha256: derivative::hash(input.get_ref()),
    entries,
  };
  storage.write(derivative::manifest_path(source), serde_json::to_vec(&manifest)?.into()).await?;
  std::fs::write(directory.join("fixture-manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;
  let jxl = &manifest.entries.iter().find(|e| e.format == F::Jxl).unwrap().path;
  let webp = &manifest.entries.iter().find(|e| e.format == F::Webp).unwrap().path;
  let html = format!(
    r#"<!doctype html><title>Production cover formats</title><style>body{{font:18px system-ui}}img{{width:384px;height:384px;background:white}}figure{{display:inline-block}}</style><h1>White cover rendering</h1><p id=ua></p><figure><figcaption>JPEG XL</figcaption><img id=jxl src="/api/static/{jxl}"></figure><figure><figcaption>WebP</figcaption><img id=webp src="/api/static/{webp}"></figure><pre id=result></pre><script>
ua.textContent=navigator.userAgent;
Promise.all(['jxl','webp'].map(async id=>{{let i=document.getElementById(id);try{{await i.decode();let c=document.createElement('canvas');c.width=i.naturalWidth;c.height=i.naturalHeight;let x=c.getContext('2d');x.drawImage(i,0,0);return {{id,complete:i.complete,width:i.naturalWidth,height:i.naturalHeight,pixel:[...x.getImageData(0,100,1,1).data]}};}}catch(e){{return {{id,error:String(e)}};}}}})).then(items=>{{result.textContent=JSON.stringify(items,null,2);fetch('/report?'+new URLSearchParams({{ua:navigator.userAgent,result:JSON.stringify(items)}}));}});
</script>"#
  );
  let ctx = Arc::new(TestingAppContext::builder().storage(storage).media(media).build()) as Arc<dyn AppContextTrait>;
  let report = directory.join("browser-reports.jsonl");
  let router = recorder::web::controller::r#static::create(ctx.clone())
    .await?
    .apply_to(Router::new())
    .with_state(ctx)
    .route(
      "/",
      get(move || {
        let html = html.clone();
        async move { Html(html) }
      }),
    )
    .route(
      "/report",
      get(move |Query(query): Query<std::collections::HashMap<String, String>>| {
        let report = report.clone();
        async move {
          use std::io::Write;
          let mut log = std::fs::OpenOptions::new().create(true).append(true).open(report).unwrap();
          writeln!(log, "{}", serde_json::to_string(&query).unwrap()).unwrap();
          http::StatusCode::NO_CONTENT
        }
      }),
    );
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  std::fs::write(directory.join("port.txt"), listener.local_addr()?.port().to_string())?;
  axum::serve(listener, router).await?;
  Ok(())
}
