use crate::bin_modules::cli::{DetailLevel, ResultType, SearchArgs};
use crate::bin_modules::database::rows::{CategoriesRow, ImagesRow};
use crate::{DestinyFetchError, Result};
use futures::TryStreamExt;
use sqlx::SqlitePool;
use sqlx::{QueryBuilder, sqlite::Sqlite};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{self, AsyncWrite, AsyncWriteExt};

pub async fn search(args: SearchArgs, conn: &SqlitePool) -> Result<()> {
    let mut wtr: Box<dyn AsyncWrite + Send + Unpin>;
    if let Some(f) = &args.output {
        wtr = Box::new(fs::File::create(f).await?);
    } else {
        wtr = Box::new(io::stdout());
    }

    if let ResultType {
        categories: true, ..
    }
    | ResultType { all: true, .. } = &args.result_type
    {
        let cat_fmt = match &args.detail_level {
            Some(DetailLevel { simple: true, .. }) => {
                move |cat: CategoriesRow| -> String { format!("{}\n", cat.title) }
            }
            Some(DetailLevel { detailed: true, .. }) => move |cat: CategoriesRow| -> String {
                format!(
                    "{:<5}, {:<30}, {:<3}, {:<3}\n",
                    cat.id, cat.title, cat.files, cat.subcats,
                )
            },
            Some(DetailLevel { default: true, .. }) | _ => move |cat: CategoriesRow| -> String {
                format!(
                    "{:<30}, images{:<4}, subcategories{:<4}\n",
                    cat.title, cat.files, cat.subcats,
                )
            },
        };
        let mut cats_q = construct_categories_query(&args);
        let mut cat_stream = cats_q.build_query_as::<'_, CategoriesRow>().fetch(conn);
        while let Ok(Some(cat)) = cat_stream.try_next().await {
            let _ = wtr.write_all(cat_fmt(cat).as_bytes()).await?;
        }
    }
    if let ResultType { images: true, .. } | ResultType { all: true, .. } = &args.result_type {
        let img_fmt = match &args.detail_level {
            Some(DetailLevel { simple: true, .. }) => {
                move |img: ImagesRow| -> String { format!("{}\n", img.title) }
            }
            Some(DetailLevel { detailed: true, .. }) => move |img: ImagesRow| -> String {
                format!(
                    "{:<5}, {:<30}, {:<5}, {:<5}, {:<10}, {:<4}, {}",
                    img.id,
                    img.title,
                    img.width,
                    img.height,
                    img.size,
                    img.ext_.map_or("NULL".to_string(), |e| e.to_string()),
                    img.url
                )
            },
            Some(DetailLevel { default: true, .. }) | _ => move |img: ImagesRow| -> String {
                format!(
                    "({:<8}) {:<30}, {:<4}x{:<4}, {:<10}KiB, {:<5}",
                    img.id,
                    img.title,
                    img.width,
                    img.height,
                    img.size,
                    img.ext_.map_or("NULL".to_string(), |e| e.to_string())
                )
            },
        };
        let mut images_q = construct_categories_query(&args);
        let mut img_stream = images_q.build_query_as::<'_, ImagesRow>().fetch(conn);
        while let Ok(Some(img)) = img_stream.try_next().await {
            let _ = wtr.write_all(img_fmt(img).as_bytes()).await?;
        }
    }

    Ok(())
}

pub fn construct_images_query(args: &SearchArgs) -> QueryBuilder<Sqlite> {
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM images ");

    q.push("WHERE LOWER(title) LIKE ");
    q.push_bind(format!("%{}% ", &args.search.to_lowercase()));

    if let Some(c) = &args.in_category {
        q.push(
            r#"AND EXISTS (
            SELECT 1
            FROM image_categories as ic
            JOIN categories ON categories.id  = ic.category_id
            WHERE ic.image_id = images.id AND categories.title = "#,
        );
        q.push_bind(c);
        q.push("\n)");
    }

    if let Some(v) = &args.ftype {
        q.push("AND extension IN (");
        let mut sep = q.separated(", ");
        for ext in v.iter() {
            sep.push_bind(ext);
        }
        sep.push_unseparated(") ");
    }

    match (&args.minsize, &args.maxsize) {
        (Some(min), Some(max)) => {
            q.push("AND size_ BETWEEN ");
            q.push_bind(min);
            q.push(" AND ");
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND size_ >= ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND size_ <= ");
            q.push_bind(max);
        }
        _ => (),
    }

    match (&args.after, &args.before) {
        (Some(min), Some(max)) => {
            q.push("AND timestamp_ BETWEEN ");
            q.push_bind(min.and_utc().timestamp());
            q.push(" AND ");
            q.push_bind(max.and_utc().timestamp());
        }
        (Some(min), None) => {
            q.push("AND timestamp_ >= ");
            q.push_bind(min.and_utc().timestamp());
        }
        (None, Some(max)) => {
            q.push("AND timestamp_ <= ");
            q.push_bind(max.and_utc().timestamp());
        }
        _ => (),
    }

    match (&args.minwidth, &args.maxwidth) {
        (Some(min), Some(max)) => {
            q.push("AND width BETWEEN ");
            q.push_bind(min);
            q.push(" AND ");
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND width >= ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND width <= ");
            q.push_bind(max);
        }
        _ => (),
    }
    match (&args.minheight, &args.maxheight) {
        (Some(min), Some(max)) => {
            q.push("AND height BETWEEN ");
            q.push_bind(min);
            q.push(" AND ");
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND height >= ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND height <= ");
            q.push_bind(max);
        }
        _ => (),
    }

    match (&args.minpixels, &args.maxpixels) {
        (Some(min), Some(max)) => {
            q.push("AND width * height BETWEEN ");
            q.push_bind(min);
            q.push(" AND ");
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND width * height >= ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND width * height <= ");
            q.push_bind(max);
        }
        _ => (),
    }

    q.push(" ORDER BY title");

    match &args.limit {
        Some(lim) if *lim >= 0_i32 => {
            q.push(" LIMIT ");
            q.push_bind(lim);
        }
        _ => (),
    }

    q
}

fn construct_categories_query(args: &SearchArgs) -> QueryBuilder<Sqlite> {
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM categories ");
    q.push("WHERE title LIKE ? ");
    q.push_bind(format!("%{}% ", args.search));
    if let Some(ic) = &args.in_category {
        q.push(
            r#"
            AND EXISTS (
                SELECT 1
                FROM subcategories as sc
                JOIN categories AS c ON sc.parent_id = c.id
                WHERE categories.id = sc.child_id AND c.title = ?
        ) "#,
        );
        q.push_bind(ic);
    }

    q.push("ORDER BY title");

    match &args.limit {
        Some(lim) if *lim >= 0 => {
            q.push(" LIMIT ?");
            q.push_bind(lim);
        }
        _ => (),
    }

    q
}

#[cfg(test)]
mod tests {
    use sqlx::Execute;

    use super::*;
    fn simple_image_search() -> SearchArgs {
        SearchArgs {
            search: "Hive".into(),
            result_type: ResultType {
                images: true,
                categories: false,
                all: false,
            },
            in_category: None,
            output: None,
            limit: None,
            detail_level: Some(DetailLevel {
                simple: true,
                default: false,
                detailed: false,
            }),
            ftype: None,
            maxsize: None,
            minsize: None,
            before: None,
            after: None,
            maxwidth: None,
            minwidth: None,
            maxheight: None,
            minheight: None,
            maxpixels: None,
            minpixels: None,
        }
    }

    #[tokio::test]
    async fn test_simple_sql1() {
        let args: SearchArgs = simple_image_search();
        let test: QueryBuilder<Sqlite> = construct_images_query(&args);
        let exp = "SELECT * FROM images WHERE title LIKE ? ORDER BY title";
        assert_eq!(test.into_string(), exp);
    }
}

// fn output_search(file: &Path, imgs: Vec<Images>, cats: Vec<Categories>) -> Result<()> {
//     let mut w = csv::Writer::from_path(file)?;

//     for img in imgs {
//         w.serialize(img)?;
//     }
//     w.flush()?;

//     for cat in cats {
//         w.serialize(cat)?;
//     }

//     Ok(())
// }

// pub fn search_images(args: &SearchArgs, conn: &SqlitePool) -> Result<Vec<Images>> {
//     match &args.result_type {
//         ResultType {
//             categories: true, ..
//         } => return Err(DestinyFetchError::WrongQueryMethod),
//         _ => (),
//     };
//     let mut q: images::BoxedQuery<'_, diesel::sqlite::Sqlite> = images::table.into_boxed();

//     // filter by contains <SEARCH> as a substring
//     q = q.filter(images::title.like(format!("%{}%", &args.search)));

//     if let Some(v) = &args.ftype {
//         let mut s: std::vec::IntoIter<String> = as_string_vec(v).into_iter();

//         let pat = format!("%{}", s.next().unwrap_or_default());
//         q = q.filter(images::title.like(pat));

//         for ext in s {
//             let n_pat = format!("%{}", &ext);
//             q = q.or_filter(images::title.like(n_pat));
//         }
//     }

//     match &args.maxsize {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::size.le(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.minsize {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::size.ge(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.maxwidth {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::width.le(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.minwidth {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::width.ge(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.maxheight {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::height.le(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.minheight {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::height.ge(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.maxpixels {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::width.mul(images::height).le(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.minpixels {
//         Some(i) if *i >= 0 => {
//             q = q.filter(images::width.mul(images::height).ge(*i));
//         }
//         Some(n) => {
//             return Err(DestinyFetchError::NegativeArgErr);
//         }
//         _ => (),
//     };

//     match &args.before {
//         Some(dt) => {
//             q = q.filter(images::timestamp_.lt(dt.naive_utc()));
//         }
//         _ => (),
//     };

//     match &args.after {
//         Some(dt) => {
//             q = q.filter(images::timestamp_.gt(dt.naive_utc()));
//         }
//         _ => (),
//     };

//     let results: Vec<Images> = match &args.in_category {
//         Some(cat) => {
//             let c: &str = match cat.strip_prefix("Category:") {
//                 Some(s) => s,
//                 None => cat.as_str(),
//             };
//             let id_: i32 = categories::table
//                 .filter(categories::title.eq(c))
//                 .select(categories::id)
//                 .first(conn)?;
//             q.inner_join(image_categories::table)
//                 .filter(image_categories::category_id.eq(id_))
//                 .select(Images::as_select())
//                 .order_by(images::title)
//                 .limit(args.limit.clone().unwrap_or(1_000_000).into())
//                 .load(conn)?
//         }
//         None => q
//             .select(Images::as_select())
//             .order_by(images::title)
//             .limit(args.limit.clone().unwrap_or(1_000_000).into())
//             .load(conn)?,
//     };

//     Ok(results)
// }

// pub fn search_categories(args: &SearchArgs, conn: &SqlitePool) -> Result<Vec<Categories>> {
//     match &args.result_type {
//         ResultType { images: true, .. } => return Err(DestinyFetchError::WrongQueryMethod),
//         _ => (),
//     };
//     let mut q: categories::BoxedQuery<'_, diesel::sqlite::Sqlite> = categories::table.into_boxed();

//     q = q.filter(categories::title.like(format!("%{}%", &args.search)));

//     let results: Vec<Categories> = match &args.in_category {
//         Some(cat) => {
//             let c: &str = match cat.strip_prefix("Category:") {
//                 Some(s) => s,
//                 None => cat.as_str(),
//             };
//             let id_: i32 = categories::table
//                 .filter(categories::title.eq(cat))
//                 .select(categories::id)
//                 .first(conn)?;
//             q.inner_join(subcategories::table.on(categories::id.eq(subcategories::subcategory_id)))
//                 .filter(subcategories::category_id.eq(id_))
//                 .select(Categories::as_select())
//                 .order_by(categories::title)
//                 .limit(args.limit.clone().unwrap_or(1_000_000).into())
//                 .load(conn)?
//         }
//         None => q
//             .select(Categories::as_select())
//             .order_by(categories::title)
//             .limit(args.limit.clone().unwrap_or(1_000_000).into())
//             .load(conn)?,
//     };

//     Ok(results)
// }
