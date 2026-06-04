use crate::bin_modules::cli::{DetailLevel, ResultType, SearchArgs};
use crate::bin_modules::{DestinyFetchError, Result};
use sqlx::{Pool, QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;

#[derive(sqlx::FromRow, derive_more::Into)]
#[into(i32)]
struct Id(i32);

#[derive(sqlx::FromRow, derive_more::Into)]
#[into(String)]
struct Title(String);

async fn validate_titles(titles: &HashSet<String>, pool: &SqlitePool) -> Result<HashSet<String>> {
    let mut tx = pool.begin().await?;
    sqlx::query!("CREATE TEMP TABLE temp_titles (title TEXT) ")
        .execute(&mut *tx)
        .await?;
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("");
    q.push("INSERT INTO temp_titles(title) ");
    q.push_values(titles.iter(), |mut b, t| {
        b.push_bind(t);
    });
    q.build().execute(&mut *tx).await?;
    q.reset();
    q.push(
        r#"
    SELECT tmp.title
    FROM temp_titles AS tmp
    LEFT JOIN images ON LOWER(tmp.title) == LOWER(images.title)
    WHERE images.title IS NOT NULL
    "#,
    );
    let valid: std::result::Result<Vec<Title>, sqlx::Error> =
        q.build_query_as().fetch_all(&mut *tx).await;
    tx.commit().await?;
    if let Err(sqlx::Error::RowNotFound) = valid {
        println!("No matching images found");
        return Err(DestinyFetchError::NoMatchingImages);
    }
    let valid_titles: HashSet<String> = valid?.into_iter().map(Into::into).collect();

    Ok(valid_titles)
}
async fn validate_ids(ids: &HashSet<i32>, pool: &SqlitePool) -> Result<HashSet<i32>> {
    let mut tx = pool.begin().await?;
    sqlx::query!("CREATE TEMP TABLE temp_ids (id INTEGER)")
        .execute(&mut *tx)
        .await?;
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("");
    q.push("INSERT INTO temp_ids(id) ");
    q.push_values(ids.iter(), |mut b, id| {
        b.push_bind(id);
    });
    q.build().execute(&mut *tx).await?;
    q.reset();
    q.push(
        r#"
    SELECT tmp.id
    FROM temp_ids AS tmp
    LEFT JOIN images ON tmp.id == images.id
    WHERE images.id IS NOT NULL
    "#,
    );
    let valid: std::result::Result<Vec<Id>, sqlx::Error> =
        q.build_query_as().fetch_all(&mut *tx).await;
    tx.commit().await?;
    if let Err(sqlx::Error::RowNotFound) = valid {
        println!("No matching images found");
        return Err(DestinyFetchError::NoMatchingImages);
    }
    let valid_ids: HashSet<i32> = valid?.into_iter().map(Into::into).collect();

    Ok(valid_ids)
}
