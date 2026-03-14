use std::collections::HashSet;

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use fetch::{HttpClientConfig, fetch_html};
use itertools::Itertools;
use lazy_static::lazy_static;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take, take_till1},
    character::complete::space1,
    combinator::map,
};
use recorder::{
    errors::{RecorderError, RecorderResult},
    extract::{
        html::extract_inner_text_from_element_ref,
        mikan::{MikanClient, MikanConfig, MikanEpisodeHash, MikanFansubHash},
    },
};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use snafu::FromString;
use url::Url;

lazy_static! {
    static ref TEST_FOLDER: std::path::PathBuf =
        if cfg!(any(test, debug_assertions, feature = "playground")) {
            std::path::PathBuf::from(format!(
                "{}/tests/resources/mikan/classic_episodes",
                env!("CARGO_MANIFEST_DIR")
            ))
        } else {
            std::path::PathBuf::from("tests/resources/mikan/classic_episodes")
        };
}

lazy_static! {
    static ref TOTAL_PAGE_REGEX: Regex =
        Regex::new(r#"\$\(\'\.classic-view-pagination2\'\)\.bootpag\(\{\s*total:\s*(\d+)"#)
            .unwrap();
}

pub struct MikanClassicEpisodeTableRow {
    pub id: i32,
    pub publish_at: DateTime<Utc>,
    pub mikan_fansub_id: Option<String>,
    pub fansub_name: Option<String>,
    pub mikan_episode_id: String,
    pub original_name: String,
    pub magnet_link: Option<String>,
    pub file_size: Option<String>,
    pub torrent_link: Option<String>,
}

impl MikanClassicEpisodeTableRow {
    fn timezone() -> FixedOffset {
        FixedOffset::east_opt(8 * 3600).unwrap()
    }

    fn fixed_date_parser(input: &str) -> IResult<&str, NaiveDate> {
        alt((
            map(tag("今天"), move |_| {
                Utc::now().with_timezone(&Self::timezone()).date_naive()
            }),
            map(tag("昨天"), move |_| {
                Utc::now().with_timezone(&Self::timezone()).date_naive() - Duration::days(1)
            }),
        ))
        .parse(input)
    }

    fn formatted_date_parser(input: &str) -> IResult<&str, NaiveDate> {
        let (remain, date_str) = take_till1(|c: char| c.is_whitespace()).parse(input)?;
        let date = NaiveDate::parse_from_str(date_str, "%Y/%m/%d").map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
        })?;
        Ok((remain, date))
    }

    fn date_parser(input: &str) -> IResult<&str, NaiveDate> {
        alt((Self::fixed_date_parser, Self::formatted_date_parser)).parse(input)
    }

    fn time_parser(input: &str) -> IResult<&str, NaiveTime> {
        let (remain, time_str) = take(5usize).parse(input)?;
        let time = NaiveTime::parse_from_str(time_str, "%H:%M").map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
        })?;
        Ok((remain, time))
    }

    fn extract_publish_at(text: &str) -> Option<DateTime<Utc>> {
        let (_, (date, _, time)) = (Self::date_parser, space1, Self::time_parser)
            .parse(text)
            .ok()?;
        let local_dt = Self::timezone()
            .from_local_datetime(&date.and_time(time))
            .single()?;
        Some(local_dt.with_timezone(&Utc))
    }

    pub fn from_element_ref(
        row: ElementRef<'_>,
        rev_id: i32,
        idx: i32,
        mikan_base_url: &Url,
    ) -> RecorderResult<Self> {
        let publish_at_selector = &Selector::parse("td:nth-of-type(1)").unwrap();
        let fansub_selector = &Selector::parse("td:nth-of-type(2) > a").unwrap();
        let original_name_selector =
            &Selector::parse("td:nth-of-type(3) > a:nth-of-type(1)").unwrap();
        let magnet_link_selector =
            &Selector::parse("td:nth-of-type(3) > a:nth-of-type(2)").unwrap();
        let file_size_selector = &Selector::parse("td:nth-of-type(4)").unwrap();
        let torrent_link_selector = &Selector::parse("td:nth-of-type(5) > a").unwrap();

        let publish_at = row
            .select(publish_at_selector)
            .next()
            .map(extract_inner_text_from_element_ref)
            .and_then(|e| Self::extract_publish_at(&e));

        let (mikan_fansub_hash, fansub_name) = row
            .select(fansub_selector)
            .next()
            .and_then(|e| {
                e.attr("href")
                    .and_then(|s| mikan_base_url.join(s).ok())
                    .and_then(|u| MikanFansubHash::from_homepage_url(&u))
                    .map(|h| (h, extract_inner_text_from_element_ref(e)))
            })
            .unzip();

        let (mikan_episode_hash, original_name) = row
            .select(original_name_selector)
            .next()
            .and_then(|el| {
                el.attr("href")
                    .and_then(|s| mikan_base_url.join(s).ok())
                    .and_then(|u| MikanEpisodeHash::from_homepage_url(&u))
                    .map(|h| (h, extract_inner_text_from_element_ref(el)))
            })
            .unzip();

        let magnet_link = row
            .select(magnet_link_selector)
            .next()
            .and_then(|el| el.attr("data-clipboard-text"));

        let file_size = row
            .select(file_size_selector)
            .next()
            .map(extract_inner_text_from_element_ref);

        let torrent_link = row
            .select(torrent_link_selector)
            .next()
            .and_then(|el| el.attr("href"));

        if let (Some(mikan_episode_hash), Some(original_name), Some(publish_at)) = (
            mikan_episode_hash.as_ref(),
            original_name.as_ref(),
            publish_at.as_ref(),
        ) {
            Ok(Self {
                id: rev_id * 1000 + idx,
                publish_at: *publish_at,
                mikan_fansub_id: mikan_fansub_hash.map(|h| h.mikan_fansub_id.clone()),
                fansub_name,
                mikan_episode_id: mikan_episode_hash.mikan_episode_id.clone(),
                original_name: original_name.clone(),
                magnet_link: magnet_link.map(|s| s.to_string()),
                file_size: file_size.map(|s| s.to_string()),
                torrent_link: torrent_link.map(|s| s.to_string()),
            })
        } else {
            let mut missing_fields = vec![];
            if mikan_episode_hash.is_none() {
                missing_fields.push("mikan_episode_id");
            }
            if original_name.is_none() {
                missing_fields.push("original_name");
            }
            if publish_at.is_none() {
                missing_fields.push("publish_at");
            }
            Err(RecorderError::without_source(format!(
                "Failed to parse episode table row, missing fields: {missing_fields:?}, row \
                 index: {idx}"
            )))
        }
    }
}

pub struct MikanClassicEpisodeTablePage {
    pub page: i32,
    pub total: i32,
    pub html: String,
    pub rows: Vec<MikanClassicEpisodeTableRow>,
}

impl MikanClassicEpisodeTablePage {
    pub fn from_html(
        html: String,
        mikan_base_url: &Url,
        page: i32,
        updated_info: Option<(i32, i32)>,
    ) -> RecorderResult<Self> {
        let tr_selector = &Selector::parse("tbody tr").unwrap();
        let doc = Html::parse_document(&html);
        if let Some(mut total) = TOTAL_PAGE_REGEX
            .captures(&html)
            .and_then(|c| c.get(1))
            .and_then(|s| s.as_str().parse::<i32>().ok())
        {
            if let Some((_, update_total)) = updated_info {
                total = update_total;
            }

            let rev_id = total - page;
            let rows = doc
                .select(tr_selector)
                .rev()
                .enumerate()
                .map(|(idx, tr)| {
                    MikanClassicEpisodeTableRow::from_element_ref(
                        tr,
                        rev_id,
                        idx as i32,
                        mikan_base_url,
                    )
                })
                .collect::<RecorderResult<Vec<_>>>()?;
            Ok(Self {
                page,
                total,
                html,
                rows,
            })
        } else {
            Err(RecorderError::without_source(
                "Failed to parse pagination meta and rows".into(),
            ))
        }
    }

    pub fn save_to_files(&self) -> RecorderResult<()> {
        use polars::prelude::*;

        let rev_id = self.total - self.page;
        let parquet_path = TEST_FOLDER.join(format!("parquet/rev_{rev_id}.parquet"));
        let csv_path = TEST_FOLDER.join(format!("csv/rev_{rev_id}.csv"));
        let html_path = TEST_FOLDER.join(format!("html/rev_{rev_id}.html"));

        std::fs::write(html_path, self.html.clone())?;

        let mut id_vec = Vec::new();
        let mut publish_at_vec = Vec::new();
        let mut mikan_fansub_id_vec = Vec::new();
        let mut fansub_name_vec = Vec::new();
        let mut mikan_episode_id_vec = Vec::new();
        let mut original_name_vec = Vec::new();
        let mut magnet_link_vec = Vec::new();
        let mut file_size_vec = Vec::new();
        let mut torrent_link_vec = Vec::new();

        for row in &self.rows {
            id_vec.push(row.id);
            publish_at_vec.push(row.publish_at.to_rfc3339());
            mikan_fansub_id_vec.push(row.mikan_fansub_id.clone());
            fansub_name_vec.push(row.fansub_name.clone());
            mikan_episode_id_vec.push(row.mikan_episode_id.clone());
            original_name_vec.push(row.original_name.clone());
            magnet_link_vec.push(row.magnet_link.clone());
            file_size_vec.push(row.file_size.clone());
            torrent_link_vec.push(row.torrent_link.clone());
        }

        let df = df! [
            "id" => id_vec,
            "publish_at_timestamp" => publish_at_vec,
            "mikan_fansub_id" => mikan_fansub_id_vec,
            "fansub_name" => fansub_name_vec,
            "mikan_episode_id" => mikan_episode_id_vec,
            "original_name" => original_name_vec,
            "magnet_link" => magnet_link_vec,
            "file_size" => file_size_vec,
            "torrent_link" => torrent_link_vec,
        ]
        .map_err(|e| {
            let message = format!("Failed to create DataFrame: {e}");
            RecorderError::with_source(Box::new(e), message)
        })?;

        let mut parquet_file = std::fs::File::create(&parquet_path)?;

        ParquetWriter::new(&mut parquet_file)
            .finish(&mut df.clone())
            .map_err(|e| {
                let message = format!("Failed to write parquet file: {e}");
                RecorderError::with_source(Box::new(e), message)
            })?;

        let mut csv_file = std::fs::File::create(&csv_path)?;

        CsvWriter::new(&mut csv_file)
            .include_header(true)
            .with_quote_style(QuoteStyle::Always)
            .finish(&mut df.clone())
            .map_err(|e| {
                let message = format!("Failed to write csv file: {e}");
                RecorderError::with_source(Box::new(e), message)
            })?;

        println!(
            "[{}/{}] Saved {} rows to rev_{}.{{parquet,html,csv}}",
            self.page,
            self.total,
            self.rows.len(),
            rev_id
        );

        Ok(())
    }

    pub fn waiting_rev_ids(total: i32) -> RecorderResult<Vec<i32>> {
        let dir = TEST_FOLDER.join("csv");

        let files = std::fs::read_dir(dir)?;

        let rev_ids = files
            .filter_map(|f| f.ok())
            .filter_map(|f| {
                f.path().file_stem().and_then(|s| {
                    s.to_str().and_then(|s| {
                        if s.starts_with("rev_") {
                            s.replace("rev_", "").parse::<i32>().ok()
                        } else {
                            None
                        }
                    })
                })
            })
            .collect::<HashSet<_>>();

        Ok((0..total)
            .filter(|rev_id| !rev_ids.contains(rev_id))
            .collect::<Vec<_>>())
    }
}

async fn scrape_mikan_classic_episode_table_page(
    mikan_client: &MikanClient,
    page: i32,
    updated_info: Option<(i32, i32)>,
) -> RecorderResult<MikanClassicEpisodeTablePage> {
    let mikan_base_url = mikan_client.base_url();
    let url = mikan_base_url.join(&format!("/Home/Classic/{page}"))?;

    if let Some((rev_id, update_total)) = updated_info.as_ref() {
        let html_path = TEST_FOLDER.join(format!("html/rev_{rev_id}.html"));
        if html_path.exists() {
            let html = std::fs::read_to_string(&html_path)?;
            println!("[{page}/{update_total}] html exists, skipping fetch");
            return MikanClassicEpisodeTablePage::from_html(
                html,
                mikan_base_url,
                page,
                updated_info,
            );
        }
    }

    let total = if let Some((_, update_total)) = updated_info.as_ref() {
        update_total.to_string()
    } else {
        "Unknown".to_string()
    };

    println!("[{page}/{total}] fetching html...");

    let html = fetch_html(mikan_client, url).await?;

    println!("[{page}/{total}] fetched html done");

    std::fs::write(TEST_FOLDER.join("html/temp.html"), html.clone())?;

    MikanClassicEpisodeTablePage::from_html(html, mikan_base_url, page, updated_info)
}

async fn scrape_mikan_classic_episode_table_page_from_rev_id(
    mikan_client: &MikanClient,
    total: i32,
    rev_idx: i32,
) -> RecorderResult<MikanClassicEpisodeTablePage> {
    let page = total - rev_idx;

    scrape_mikan_classic_episode_table_page(mikan_client, page, Some((rev_idx, total))).await
}

async fn merge_mikan_classic_episodes_and_strip_columns() -> RecorderResult<()> {
    use polars::prelude::*;

    let dir = TEST_FOLDER.join("parquet");
    let files = std::fs::read_dir(dir)?;

    let parquet_paths = files
        .filter_map(|f| f.ok())
        .filter_map(|f| {
            let path = f.path();
            if let Some(ext) = path.extension()
                && ext == "parquet"
                && path
                    .file_stem()
                    .is_some_and(|f| f.to_string_lossy().starts_with("rev_"))
            {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if parquet_paths.is_empty() {
        return Err(RecorderError::without_source(
            "No parquet files found to merge".into(),
        ));
    }

    println!("Found {} parquet files to merge", parquet_paths.len());

    // 读取并合并所有 parquet 文件
    let mut all_dfs = Vec::new();
    for path in &parquet_paths {
        println!("Reading {path:?}");
        let file = std::fs::File::open(path)?;
        let df = ParquetReader::new(file).finish().map_err(|e| {
            let message = format!("Failed to read parquet file {path:?}: {e}");
            RecorderError::with_source(Box::new(e), message)
        })?;
        all_dfs.push(df);
    }

    let lazy_frames: Vec<LazyFrame> = all_dfs.into_iter().map(|df| df.lazy()).collect();

    let merged_df = concat_lf_diagonal(&lazy_frames, UnionArgs::default())
        .map_err(|e| {
            let message = format!("Failed to concat DataFrames: {e}");
            RecorderError::with_source(Box::new(e), message)
        })?
        .sort(
            ["publish_at_timestamp"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .unique(
            Some(cols(["mikan_fansub_id", "mikan_episode_id"])),
            UniqueKeepStrategy::First,
        )
        .collect()
        .map_err(|e| {
            let message = format!("Failed to collect lazy DataFrame: {e}");
            RecorderError::with_source(Box::new(e), message)
        })?;

    fn select_columns_and_write(
        merged_df: DataFrame,
        name: &str,
        columns: &[&str],
    ) -> RecorderResult<()> {
        let result_df = merged_df
            .lazy()
            .sort(["publish_at_timestamp"], SortMultipleOptions::default())
            .select(columns.iter().map(|c| col(*c)).collect_vec())
            .collect()
            .map_err(|e| {
                let message = format!("Failed to sort and select columns: {e}");
                RecorderError::with_source(Box::new(e), message)
            })?;

        let output_path = TEST_FOLDER.join(format!("parquet/{name}.parquet"));
        let mut output_file = std::fs::File::create(&output_path)?;

        ParquetWriter::new(&mut output_file)
            .set_parallel(true)
            .with_compression(ParquetCompression::Zstd(None))
            .finish(&mut result_df.clone())
            .map_err(|e| {
                let message = format!("Failed to write merged parquet file: {e}");
                RecorderError::with_source(Box::new(e), message)
            })?;

        println!("Merged {} rows into {output_path:?}", result_df.height());
        Ok(())
    }

    select_columns_and_write(merged_df.clone(), "tiny", &["fansub_name", "original_name"])?;
    // select_columns_and_write(
    //     merged_df.clone(),
    //     "lite",
    //     &[
    //         "mikan_fansub_id",
    //         "fansub_name",
    //         "mikan_episode_id",
    //         "original_name",
    //     ],
    // )?;
    // select_columns_and_write(
    //     merged_df,
    //     "full",
    //     &[
    //         "id",
    //         "publish_at_timestamp",
    //         "mikan_fansub_id",
    //         "fansub_name",
    //         "mikan_episode_id",
    //         "original_name",
    //         "magnet_link",
    //         "file_size",
    //         "torrent_link",
    //     ],
    // )?;

    Ok(())
}

#[tokio::main]
async fn main() -> RecorderResult<()> {
    std::fs::create_dir_all(TEST_FOLDER.join("html"))?;
    std::fs::create_dir_all(TEST_FOLDER.join("parquet"))?;
    std::fs::create_dir_all(TEST_FOLDER.join("csv"))?;

    let mikan_scrape_client = MikanClient::from_config(MikanConfig {
        http_client: HttpClientConfig {
            exponential_backoff_max_retries: Some(3),
            leaky_bucket_max_tokens: Some(2),
            leaky_bucket_initial_tokens: Some(1),
            leaky_bucket_refill_tokens: Some(1),
            leaky_bucket_refill_interval: Some(std::time::Duration::from_millis(1000)),
            user_agent: Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0"
                    .to_string(),
            ),
            ..Default::default()
        },
        base_url: Url::parse("https://mikanani.me")?,
    })
    .await?;

    let first_page_and_pagination_info =
        scrape_mikan_classic_episode_table_page(&mikan_scrape_client, 1, None).await?;

    let total_page = first_page_and_pagination_info.total;

    first_page_and_pagination_info.save_to_files()?;

    let next_rev_ids = MikanClassicEpisodeTablePage::waiting_rev_ids(total_page)?;

    for todo_rev_id in next_rev_ids {
        let page = scrape_mikan_classic_episode_table_page_from_rev_id(
            &mikan_scrape_client,
            total_page,
            todo_rev_id,
        )
        .await?;

        page.save_to_files()?;
    }

    // 合并所有 parquet 文件
    println!("\nMerging all parquet files...");

    merge_mikan_classic_episodes_and_strip_columns().await?;

    println!("Merge completed!");

    Ok(())
}
