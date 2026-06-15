use super::input::*;
use super::interactive::{promp_download_input, prompt_confirm_download, prompt_output_dir};
use crate::bin_modules::cli::{DetailLevel, DownloadArgs, FileInput, ResultType, StdinInput};
use crate::bin_modules::database::rows::ImagesRow;
use crate::bin_modules::{DestinyFetchError, Result};
use sqlx::{Pool, QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;
use std::path::PathBuf;

pub async fn download(args: DownloadArgs, conn: SqlitePool) -> Result<()> {
    if args.no_input() {
        // empty download args => interctive download
        match args.noconfirm {
            true => return Err(DestinyFetchError::MissingArgErr),
            false => {
                args = promp_download_input()?;
            }
        }
    }
    let mut rows: Vec<ImagesRow> = vec![];
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
            if let Some(save) = f.from_cached {
                jset.spawn(parse_cached_search(save));
            }
        }
        _ => return Err(DestinyFetchError::MissingArgErr),
    }
    for r in jset.join_next().await {
        rows.extend(r??);
    }
    let path = match args {
        DownloadArgs {
            output_dir: Some(path),
            ..
        } => path,
        DownloadArgs {
            output_dir: None,
            noconfirm: false,
            ..
        } => prompt_output_dir()?,
        _ => std::env::current_dir()?,
    };
    if !&args.noconfirm && !prompt_confirm_download(&rows[..], &path) {
        return Err(DestinyFetchError::Quit);
    }

    Ok(())
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
