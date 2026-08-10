use super::input::*;
use super::interactive::{promp_download_input, prompt_confirm_download, prompt_output_dir};
use crate::bin_modules::cli::{DetailLevel, DownloadArgs, FileInput, ResultType, StdinInput};
use crate::bin_modules::database::rows::{Ext, ImagesRow};
use crate::bin_modules::{DestinyFetchError, Result};
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use iter_chunks::IterChunks;
use sqlx::{Pool, QueryBuilder, Sqlite, SqlitePool};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn download(mut args: DownloadArgs, conn: SqlitePool) -> Result<()> {
    if args.no_input() {
        // empty download args => interctive download
        match args.noconfirm {
            true => return Err(DestinyFetchError::MissingArgErr),
            false => {
                args = promp_download_input()?;
            }
        }
    }
    let mut jset: tokio::task::JoinSet<Result<Vec<ImagesRow>>> = tokio::task::JoinSet::new();
    match args {
        DownloadArgs { input: Some(i), .. } => {
            if let Some(t) = i.titles {
                jset.spawn(validate_titles(t, conn.clone()));
            }
            if let Some(id) = i.ids {
                let casted: Vec<i32> = id.into_iter().map(|n| n as i32).collect();
                jset.spawn(validate_ids(casted, conn.clone()));
            }
            if let Some(c) = i.in_category {
                jset.spawn(validate_category(c, conn.clone()));
            }
        }
        DownloadArgs {
            input_file: Some(f),
            ..
        } => {
            if let Some(t) = f.titles_input {
                let titles = parse_titles_file(t).await?;
                jset.spawn(validate_titles(titles, conn.clone()));
            }
            if let Some(id) = f.ids_input {
                let ids = parse_ids_file(id).await?;
                jset.spawn(validate_ids(ids, conn.clone()));
            }
        }
        _ => return Err(DestinyFetchError::MissingArgErr),
    }
    let mut images: Vec<ImagesRow> = vec![];
    while let Some(r) = jset.join_next().await {
        match r {
            Ok(Err(DestinyFetchError::Quit | DestinyFetchError::Unknown)) => {
                tracing::error!("Shutting down program");
                return Err(DestinyFetchError::Quit);
            }
            Ok(Err(e)) => {
                return Err(e);
            }
            Ok(Ok(v)) => {
                images.extend(v);
            }
            Err(e) => {
                return Err(e)?;
            }
        }
    }
    let mut images_remaining: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(images.len()));
    let mut path = match args {
        DownloadArgs {
            output_dir: Some(path),
            ..
        } => {
            tracing::debug!("Output directory passed explicitly");
            path
        }
        DownloadArgs {
            output_dir: None,
            noconfirm: false,
            ..
        } => {
            tracing::debug!("No output directory specified, prompting user...");
            prompt_output_dir()?
        }
        _ => {
            tracing::warn!(
                "No output directory specified for a non-interactive run, using workind directory"
            );
            std::env::current_dir()?
        }
    };
    tracing::debug!(output_directory = ?path);
    if !&args.noconfirm && !prompt_confirm_download(&images[..], &path) {
        tracing::error!("User chose to quit, shutting down program");
        return Err(DestinyFetchError::Quit);
    }
    let mut mbar = MultiProgress::new();
    let mut chunk_size: usize = 4;
    while images.len() / chunk_size > 10 {
        chunk_size *= 2;
    }
    tracing::debug!(images_per_worker = ?chunk_size);
    let mut jset: tokio::task::JoinSet<Result<()>> = tokio::task::JoinSet::new();
    let mut chunks = images.into_iter().chunks(chunk_size);
    let mut count: usize = 0;
    while let Some(chunk) = chunks.next() {
        let chunk: Vec<ImagesRow> = chunk.collect();
        let total_bytes: u64 = chunk
            .iter()
            .map(|i| i.size * 1024)
            .sum::<i64>()
            .try_into()
            .unwrap_or(0_u64);

        let barstyle = ProgressStyle::with_template(
            " [{elapsed_precise}] {bar:40.green/blue} {binary_bytes}/{binary_total_bytes}",
        )
        .map_err(|_| DestinyFetchError::Unknown)?;
        let pbar: ProgressBar =
            mbar.insert(count, ProgressBar::new(total_bytes).with_style(barstyle));
        let dir = path.clone();
        let remaining: Arc<AtomicUsize> = images_remaining.clone();
        let wid = jset
            .spawn(fetch_and_save_images(chunk, dir, pbar, remaining))
            .id();
        count += 1;
        tracing::debug!(
            worker_id = ?wid,
            "worker # {} spawned, {} bytes to process",
            count,
            total_bytes
        );
    }
    while let Some(result) = jset.join_next_with_id().await {
        match result? {
            (id, Ok(_)) => {
                tracing::debug!(worker_id = ?id, "Collected worker");
            }
            (id, Err(DestinyFetchError::Quit)) => {
                tracing::error!(worker_id = ?id, "Quit signal recieved, shutting down program");
                return Err(DestinyFetchError::Quit);
            }
            (id, Err(e)) => {
                tracing::error!(worker_id = ?id, err = ?e);
                return Err(e);
            }
        }
    }
    debug_assert_eq!(images_remaining.load(Ordering::Relaxed), 0);

    Ok(())
}

pub async fn fetch_and_save_images(
    images: Vec<ImagesRow>,
    mut path: PathBuf,
    progbar: ProgressBar,
    total_images_remaining: Arc<AtomicUsize>,
) -> Result<()> {
    for image in images.iter() {
        let fname: PathBuf = filename_from_title(&image.title, &image.extension);
        path.set_file_name(fname.as_path());
        tracing::debug!(filepath = ?path);
        progbar.set_message(fname.to_string_lossy().into_owned());
        let mut f = File::create(&path).await?;
        let mut resp_stream = reqwest::get(image.url.trim()).await?.bytes_stream();
        tracing::debug!(url = ?image.url.trim());
        while let Some(chunk) = resp_stream.next().await {
            let bytes = chunk?;
            f.write_all(&bytes).await?;
            let n_bytes: Option<u64> = bytes.len().try_into().ok();
            if let Some(n) = n_bytes {
                tracing::debug!("bytes written {}", n);
                progbar.inc(n);
            }
        }
        tracing::debug!("Downloaded image {}", fname.display());
        total_images_remaining.fetch_sub(1, Ordering::Acquire);
    }
    progbar.finish_and_clear();
    Ok(())
}

pub fn filename_from_title(title_str: &String, ext: &Ext) -> PathBuf {
    let mut title: String = title_str.trim().replace(" ", "_");
    if let Some(i) = title.find('.') {
        title = title[..i].into();
    }
    let mut path = PathBuf::from(title);
    if !matches!(ext, Ext::UNKNOWN) {
        path.set_extension(ext.to_string());
    }
    path
}

pub fn get_download_size(images: &[ImagesRow]) -> u64 {
    images
        .iter()
        .filter_map(|img| {
            let b: Option<u64> = (img.size * 1024).try_into().ok();
            b
        })
        .sum()
}

pub async fn validate_titles(titles: Vec<String>, pool: SqlitePool) -> Result<Vec<ImagesRow>> {
    let mut tx = pool.begin().await?;
    sqlx::query!("CREATE TEMP TABLE temp_titles (title TEXT PRIMARY KEY) ")
        .execute(&mut *tx)
        .await?;
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("");
    q.push("INSERT OR IGNORE INTO temp_titles(title) ");
    q.push_values(titles.iter(), |mut b, t| {
        b.push_bind(t);
    });
    q.build().execute(&mut *tx).await?;
    q.reset();
    q.push(
        r#"
    SELECT images.*
    FROM temp_titles AS tmp
    LEFT JOIN images ON LOWER(tmp.title) == LOWER(images.title)
    WHERE images.title IS NOT NULL
    "#,
    );
    let valid: std::result::Result<Vec<ImagesRow>, sqlx::Error> =
        q.build_query_as().fetch_all(&mut *tx).await;
    tx.commit().await?;
    match valid {
        Err(sqlx::Error::RowNotFound) => {
            println!("No matching images found");
            return Err(DestinyFetchError::NoMatchingImages);
        }
        Err(e) => Err(e.into()),
        Ok(v) => Ok(v),
    }
}
pub async fn validate_ids(ids: Vec<i32>, pool: SqlitePool) -> Result<Vec<ImagesRow>> {
    let mut tx = pool.begin().await?;
    sqlx::query!("CREATE TEMP TABLE temp_ids (id INTEGER PRIMARY KEY)")
        .execute(&mut *tx)
        .await?;
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("");
    q.push("INSERT OR IGNORE INTO temp_ids(id) ");
    q.push_values(ids.iter(), |mut b, id| {
        if *id >= 0 {
            b.push_bind(id);
        }
    });
    q.build().execute(&mut *tx).await?;
    q.reset();
    q.push(
        r#"
    SELECT images.*
    FROM temp_ids AS tmp
    LEFT JOIN images ON tmp.id == images.id
    WHERE images.id IS NOT NULL
    "#,
    );
    let valid: std::result::Result<Vec<ImagesRow>, sqlx::Error> =
        q.build_query_as().fetch_all(&mut *tx).await;
    tx.commit().await?;
    match valid {
        Err(sqlx::Error::RowNotFound) => {
            println!("No matching images found");
            return Err(DestinyFetchError::NoMatchingImages);
        }
        Err(e) => Err(e.into()),
        Ok(v) => Ok(v),
    }
}

pub async fn validate_category(category: String, pool: SqlitePool) -> Result<Vec<ImagesRow>> {
    let mut query: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
    SELECT * FROM categories WHERE EXISTS (
        SELECT 1
        FROM subcategories as sc
        JOIN categories AS c ON sc.parent_id = c.id
        WHERE categories.id = sc.child_id AND c.title = "#,
    );
    query.push_bind(category);
    query.push(" )");
    let rows: std::result::Result<Vec<ImagesRow>, sqlx::Error> =
        query.build_query_as().fetch_all(&pool).await;
    match rows {
        Err(sqlx::Error::RowNotFound) => {
            println!("No matching images found");
            return Err(DestinyFetchError::NoMatchingImages);
        }
        Err(e) => Err(e.into()),
        Ok(v) => Ok(v),
    }
}
