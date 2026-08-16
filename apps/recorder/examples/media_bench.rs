//! Offline fixed-preset measurements; production never scores or searches.
#[path = "../tests/integration/support/jxl_decode.rs"]
mod jxl_support;
use std::{
  io::Cursor,
  path::{Path, PathBuf},
  process::Command,
  time::Instant,
};

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use recorder::media::{MediaService, decode, derivative::hash, normalize_cover};
use serde_json::{Value, json};
fn text_pixel(x: u32, y: u32) -> bool {
  let glyph = match b"JXL WEBP 2026"[((x / 18) % 13) as usize] {
    b'J' => [7, 2, 2, 2, 18, 18, 12],
    b'X' => [17, 17, 10, 4, 10, 17, 17],
    b'L' => [16, 16, 16, 16, 16, 16, 31],
    b'W' => [17, 17, 17, 21, 21, 21, 10],
    b'E' => [31, 16, 16, 30, 16, 16, 31],
    b'B' => [30, 17, 17, 30, 17, 17, 30],
    b'P' => [30, 17, 17, 30, 16, 16, 16],
    b'2' => [14, 17, 1, 2, 4, 8, 31],
    b'0' => [14, 17, 19, 21, 25, 17, 14],
    b'6' => [14, 16, 16, 30, 17, 17, 14],
    _ => [0; 7],
  };
  let (row, column) = ((y % 28) / 3, (x % 18) / 3);
  row < 7 && column < 5 && glyph[row as usize] & (1 << (4 - column)) != 0
}

fn png(image: &DynamicImage) -> Vec<u8> {
  let mut bytes = Cursor::new(Vec::new());
  image.write_to(&mut bytes, ImageFormat::Png).unwrap();
  bytes.into_inner()
}
fn composite(image: &RgbaImage, bg: f32) -> DynamicImage {
  DynamicImage::ImageRgb8(image::RgbImage::from_fn(image.width(), image.height(), |x, y| {
    let p = image.get_pixel(x, y);
    let a = f32::from(p[3]) / 255.;
    image::Rgb([0, 1, 2].map(|c| (f32::from(p[c]) * a + bg * 255. * (1. - a)).round() as u8))
  }))
}
fn metric(tool: &str, source: &Path, target: &Path) -> f64 {
  let result = Command::new(tool).arg(source).arg(target).output().unwrap();
  assert!(result.status.success(), "Official scorer failed");
  String::from_utf8(result.stdout).unwrap().trim().parse().unwrap()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).with_ansi(false).init();
  let output = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "temp/iteration6/bench".into()));
  std::fs::create_dir_all(&output)?;
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
  let config = recorder::media::MediaConfig::default();
  let media = MediaService::from_config(config.clone()).await?;
  if std::env::args().nth(2).as_deref() == Some("--resources-only") {
    let mut measurements = Vec::new();
    for name in ["076c1094.jpg", "one-mp", "upper-rgb", "upper-rgba", "transparent-resize"] {
      let source = if name.ends_with(".jpg") {
        root.join(format!("resources/mikan/doppel/images/Bangumi/202504/{name}"))
      } else {
        output.join(format!("{name}.png"))
      };
      let raw: bytes::Bytes = std::fs::read(source)?.into();
      let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
      let stop = done.clone();
      let monitor = std::thread::spawn(move || {
        let mut system = sysinfo::System::new();
        let pid = sysinfo::get_current_pid().unwrap();
        let mut peak = 0;
        while !stop.load(std::sync::atomic::Ordering::Acquire) {
          system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            sysinfo::ProcessRefreshKind::nothing().with_memory(),
          );
          peak = peak.max(system.process(pid).map_or(0, |p| p.memory()));
          std::thread::sleep(std::time::Duration::from_millis(5));
        }
        peak
      });
      println!("resource-start {name}");
      media.optimize_cover_to_webp(raw.clone(), None).await?;
      media.optimize_cover_to_jxl(raw.clone(), None).await?;
      let mut repetitions = Vec::new();
      for _ in 0..5 {
        let start = Instant::now();
        media.optimize_cover_to_webp(raw.clone(), None).await?;
        let webp = start.elapsed().as_secs_f64();
        media.optimize_cover_to_jxl(raw.clone(), None).await?;
        repetitions.push(json!({"webp_seconds": webp, "pipeline_seconds": start.elapsed().as_secs_f64()}));
      }
      done.store(true, std::sync::atomic::Ordering::Release);
      let parent_peak_rss = monitor.join().unwrap();
      measurements.push(json!({"name":name, "warmup":1, "repetitions":repetitions, "parent_peak_rss_bytes":parent_peak_rss, "sample_ms":5}));
      std::fs::write(output.join("resources.json"), serde_json::to_vec_pretty(&measurements)?)?;
      println!("resource-finish {name} parent_peak_rss_bytes={parent_peak_rss}");
    }
    return Ok(());
  }
  let djxl = std::env::var("RECORDER_BENCH_DJXL")?;
  let scorer = std::env::var("RECORDER_BENCH_SSIMULACRA2")?;
  let mut paths = std::fs::read_dir(root.join("resources/mikan/doppel/images/Bangumi/202504"))?
    .map(|e| e.unwrap().path())
    .filter(|p| p.extension().is_some_and(|e| e == "jpg"))
    .collect::<Vec<_>>();
  paths.sort();
  paths.truncate(40);
  let mut samples = paths
    .iter()
    .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), p.clone(), "cover"))
    .collect::<Vec<_>>();
  for (quarter, name) in [
    ("202204", "d8ef46c0"),
    ("202407", "997f06af"),
    ("202501", "424750fe"),
    ("202309", "5ce9fed1"),
    ("202501", "d5a4b73b"),
    ("202501", "2e430a10"),
    ("202501", "c63dd1b9"),
    ("202410", "5affa567"),
  ] {
    samples.push((
      format!("{quarter}-{name}.jpg"),
      root.join(format!("resources/mikan/doppel/images/Bangumi/{quarter}/{name}.jpg")),
      "cover",
    ));
  }
  assert_eq!(samples.len(), 48);
  for kind in ["text", "lines", "transparent", "gradient"] {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(512, 512, |x, y| {
      let v = match kind {
        "text" => {
          if text_pixel(x, y) {
            0
          } else {
            255
          }
        }
        "lines" => {
          if x % 31 < 2 || y % 29 < 2 {
            0
          } else {
            255
          }
        }
        _ => ((x + y) / 4) as u8,
      };
      Rgba([
        v,
        if kind == "gradient" { (x / 2) as u8 } else { v },
        if kind == "gradient" { (y / 2) as u8 } else { v },
        if kind == "transparent" { (x / 2) as u8 } else { 255 },
      ])
    }));
    let path = output.join(format!("generated-{kind}.png"));
    std::fs::write(&path, png(&image))?;
    samples.push((format!("generated-{kind}"), path, "synthetic"));
  }
  let mut mosaic = RgbaImage::new(2048, 3072);
  for index in 0..24 {
    let bytes = std::fs::read(&paths[index % 20])?;
    let tile = decode(&bytes, &config)?
      .resize_exact(512, 512, image::imageops::FilterType::Lanczos3)
      .into_rgba8();
    image::imageops::replace(&mut mosaic, &tile, ((index % 4) * 512) as i64, ((index / 4) * 512) as i64);
  }
  let mosaic_path = output.join("mosaic.png");
  std::fs::write(&mosaic_path, png(&DynamicImage::ImageRgba8(mosaic)))?;

  for (name, w, h, alpha) in [
    ("one-mp", 1024, 1024, false),
    ("upper-rgb", 1600, 1600, false),
    ("upper-rgba", 1600, 1600, true),
    ("transparent-resize", 2048, 2048, true),
  ] {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| {
      Rgba([
        ((x * 17 + y * 5) % 256) as u8,
        ((x * 3 + y * 11) % 256) as u8,
        ((x * 7 + y * 13) % 256) as u8,
        if alpha { ((x + y) % 256) as u8 } else { 255 },
      ])
    }));
    let image = if alpha { image } else { DynamicImage::ImageRgb8(image.into_rgb8()) };
    let path = output.join(format!("{name}.png"));
    std::fs::write(&path, png(&image))?;
    samples.push((name.into(), path, "resource"));
  }
  let mut records = Vec::new();
  for (name, path, kind) in samples {
    let raw: bytes::Bytes = std::fs::read(&path)?.into();
    let source = normalize_cover(decode(&raw, &config)?, &config)?.into_rgba8();
    let work = output.join(&name);
    std::fs::create_dir_all(&work)?;
    let normalized = work.join("source.png");
    DynamicImage::ImageRgba8(source.clone()).save(&normalized)?;
    let start = Instant::now();
    let webp = media.optimize_cover_to_webp(raw.clone(), None).await?;
    let webp_seconds = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let jxl = media.optimize_cover_to_jxl(raw.clone(), None).await?;
    let jxl_seconds = start.elapsed().as_secs_f64();
    std::fs::write(work.join("encoded.jxl"), &jxl)?;
    std::fs::write(work.join("baseline.webp"), &webp)?;
    let decoded = jxl_support::decode(&jxl);
    assert_eq!(decoded.dimensions(), source.dimensions());
    assert!(decoded.pixels().zip(source.pixels()).all(|(a, b)| a[3] == b[3]));
    let pixels = work.join("decoded.png");
    DynamicImage::ImageRgba8(decoded.clone()).save(&pixels)?;
    let cross = Command::new(&djxl)
      .arg(work.join("encoded.jxl"))
      .arg(work.join("djxl.png"))
      .arg("--num_threads=1")
      .output()?;
    assert!(cross.status.success());
    let cross = image::open(work.join("djxl.png"))?.into_rgba8();
    assert_eq!(cross.dimensions(), source.dimensions());
    assert!(cross.pixels().zip(source.pixels()).all(|(a, b)| a[3] == b[3]));
    let webp_pixels = image::load_from_memory(&webp)?.into_rgba8();
    assert_eq!(webp_pixels.dimensions(), source.dimensions());
    assert!(webp_pixels.pixels().zip(source.pixels()).all(|(a, b)| a[3] == b[3]));
    let mut scores = Vec::new();
    for bg in [0., 1.] {
      let src = work.join(format!("reference-{bg}.png"));
      let jc = work.join(format!("jxl-{bg}.png"));
      let wc = work.join(format!("webp-{bg}.png"));
      composite(&source, bg).save(&src)?;
      composite(&decoded, bg).save(&jc)?;
      composite(&webp_pixels, bg).save(&wc)?;
      scores.push(json!({"background":bg,"jxl":metric(&scorer,&src,&jc),"webp":metric(&scorer,&src,&wc)}));
      if source.pixels().all(|p| p[3] == 255) {
        break;
      }
    }
    let mut timing = Vec::new();
    if kind == "resource" || name == "076c1094.jpg" {
      // Bench-only repetition measures the fixed recipe; production encodes
      // once.
      media.optimize_cover_to_webp(raw.clone(), None).await?;
      media.optimize_cover_to_jxl(raw.clone(), None).await?;
      for _ in 0..5 {
        let start = Instant::now();
        media.optimize_cover_to_webp(raw.clone(), None).await?;
        let w = start.elapsed().as_secs_f64();
        media.optimize_cover_to_jxl(raw.clone(), None).await?;
        timing.push(json!({"webp_seconds":w,"pipeline_seconds":start.elapsed().as_secs_f64()}));
      }
    }
    records.push(json!({"name":name,"kind":kind,"source":path,"source_sha256":hash(&raw),"normalized_sha256":hash(&std::fs::read(normalized)?),"width":source.width(),"height":source.height(),"jxl_sha256":hash(&jxl),"webp_sha256":hash(&webp),"jxl_bytes":jxl.len(),"webp_bytes":webp.len(),"ratio":jxl.len() as f64/webp.len() as f64,"alpha_exact":true,"scores":scores,"initial_jxl_pipeline_seconds":jxl_seconds,"initial_webp_pipeline_seconds":webp_seconds,"warmup":if timing.is_empty(){0}else{1},"repetitions":timing}));
    std::fs::write(output.join("results.json"), serde_json::to_vec_pretty(&records)?)?;
    println!("sample {name}: {} / {} bytes", jxl.len(), webp.len());
  }
  let covers = records.iter().filter(|x| x["kind"] == "cover").collect::<Vec<_>>();
  let jxl: u64 = covers.iter().map(|x| x["jxl_bytes"].as_u64().unwrap()).sum();
  let webp: u64 = covers.iter().map(|x| x["webp_bytes"].as_u64().unwrap()).sum();
  let summary: Value = json!({"encoder":"jpxl 0.3.0","rev":"1e2004aa0672259279cc24a52d4b8e264f675b77","quality":77,"effort":"Balanced","threads":1,"progressive":false,"white_srgb":true,"max_cover_dimension":1600,"samples":records.len(),"fixed_samples":records.iter().filter(|r|r["kind"]!="resource").count(),"resource_samples":records.iter().filter(|r|r["kind"]=="resource").count(),"covers":covers.len(),"cover_jxl_bytes":jxl,"cover_webp_bytes":webp,"cover_ratio":jxl as f64/webp as f64,"per_image_size_fallback":false,"default":"automatic JXL + WebP"});
  std::fs::write(output.join("summary.json"), serde_json::to_vec_pretty(&summary)?)?;
  println!("{summary}");
  Ok(())
}
