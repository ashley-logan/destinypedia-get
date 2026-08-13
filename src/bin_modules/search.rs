use crate::bin_modules::cli::{DetailLevel, ResultType, SearchArgs};
use crate::bin_modules::database::rows::{CategoriesRow, ImagesRow};
use crate::{DestinyFetchError, Result};
use futures::TryStreamExt;
use sqlx::Pool;
use sqlx::query::QueryAs;
use sqlx::{QueryBuilder, sqlite::Sqlite};
use std::path::{Path, PathBuf};
use std::{fs, io};

fn write_categories(
    wtr: &mut Box<dyn std::io::Write>,
    categories: &[CategoriesRow],
    format_detail: &Option<DetailLevel>,
) -> Result<()> {
    match format_detail {
        Some(DetailLevel { titles: true, .. }) => {
            for row in categories {
                writeln!(wtr, "{}", row.title)?;
            }
        }
        Some(DetailLevel { ids: true, .. }) => {
            for row in categories {
                writeln!(wtr, "{}", row.id)?;
            }
        }
        Some(DetailLevel { detailed: true, .. }) => {
            for row in categories {
                writeln!(
                    wtr,
                    "{:<5}, {:<30}, {:<3}, {:<3}",
                    row.id, row.title, row.files, row.subcats
                )?;
            }
        }
        Some(DetailLevel { default: true, .. }) | _ => {
            for row in categories {
                writeln!(
                    wtr,
                    "{:<30}, images{:<4}, subcategories{:<4}",
                    row.title, row.files, row.subcats
                )?;
            }
        }
    }

    Ok(())
}

fn write_images(
    wtr: &mut Box<dyn std::io::Write>,
    images: &[ImagesRow],
    format_detail: &Option<DetailLevel>,
) -> Result<()> {
    match format_detail {
        Some(DetailLevel { titles: true, .. }) => {
            for row in images {
                writeln!(wtr, "{}", row.id)?;
            }
        }
        Some(DetailLevel { ids: true, .. }) => {
            for row in images {
                writeln!(wtr, "{}", row.id)?;
            }
        }
        Some(DetailLevel { detailed: true, .. }) => {
            for row in images {
                writeln!(
                    wtr,
                    "{:<5}, {:<30}, {:<5}, {:<5}, {:<10}, {:<4}, {}",
                    row.id, row.title, row.width, row.height, row.size, row.extension, row.url
                )?;
            }
        }
        Some(DetailLevel { default: true, .. }) | _ => {
            for row in images {
                writeln!(
                    wtr,
                    "({:<8}) {:<30}, {:<4}x{:<4}, {:<10}KiB, {:<5}",
                    row.id, row.title, row.width, row.height, row.size, row.extension
                )?;
            }
        }
    }
    Ok(())
}

pub async fn search(args: &SearchArgs, conn: Pool<Sqlite>) -> Result<()> {
    let mut wtr: Box<dyn io::Write>; // creates the writer that will wriite the results of the search

    if let Some(f) = &args.output {
        wtr = Box::new(fs::File::create(f)?); // write to the newly created filepath
    } else {
        wtr = Box::new(io::stdout()); // write to stdout
    }

    // if the search results will include the names of Categories, then create the
    // category result format according to the user's preferences
    let mut category_results: Option<Vec<CategoriesRow>> = None;
    if let ResultType {
        categories: true, ..
    }
    | ResultType { all: true, .. } = &args.result_type
    {
        category_results = fetch_categories(&args, conn.clone()).await?;
    }

    // if the search results will include the names of Images, then create the
    // image result format according to the user's preferences
    let mut image_results: Option<Vec<ImagesRow>> = None;
    if let ResultType { images: true, .. } | ResultType { all: true, .. } = &args.result_type {
        image_results = fetch_images(&args, conn.clone()).await?;
    }

    if let Some(v) = category_results {
        write_categories(&mut wtr, &v[..], &args.detail_level)?;
    }

    if let Some(v) = image_results {
        write_images(&mut wtr, &v[..], &args.detail_level)?;
    }

    Ok(())
}

pub async fn fetch_images(args: &SearchArgs, pool: Pool<Sqlite>) -> Result<Option<Vec<ImagesRow>>> {
    let search: String = format!("%{}%", args.search);
    let lim = args.limit.unwrap_or(500);
    let qry_result = sqlx::query_as!(
        ImagesRow,
        " \
        SELECT * FROM images WHERE LOWER(title) LIKE $1 \
        AND ($2 IS NULL OR EXISTS ( \
        SELECT 1 FROM image_categories AS ic \
        JOIN categories ON categories.id  = ic.category_id \
        WHERE ic.image_id = images.id AND categories.title = $2 \
        )) \
        AND ($3 IS NULL OR size >= $3) \
        AND ($4 IS NULL OR size <= $4) \
        AND ($5 IS NULL OR timestamp >= $5) \
        AND ($6 IS NULL OR timestamp <= $6) \
        AND ($7 IS NULL OR width >= $7) \
        AND ($8 IS NULL OR width <= $8) \
        AND ($9 IS NULL OR height >= $9) \
        AND ($10 IS NULL OR height <= $10) \
        AND ($11 IS NULL OR width * height >= $11) \
        AND ($12 IS NULL OR width * height <= $12) \
        ORDER BY title \
        LIMIT $13
        ",
        search,
        args.in_category,
        args.minsize,
        args.maxsize,
        args.after,
        args.before,
        args.minwidth,
        args.maxwidth,
        args.minheight,
        args.maxheight,
        args.minpixels,
        args.maxpixels,
        lim
    )
    .fetch_all(&pool)
    .await;

    match qry_result {
        Ok(v) if v.is_empty() => Ok(None),
        Ok(v) => Ok(Some(v)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(DestinyFetchError::SqlxErr(e)),
    }
}

pub async fn fetch_categories(
    args: &SearchArgs,
    pool: Pool<Sqlite>,
) -> Result<Option<Vec<CategoriesRow>>> {
    let search: String = format!("%{}%", args.search);
    let qry_result = sqlx::query_as!(
        CategoriesRow,
        " \
        SELECT * FROM categories WHERE LOWER(title) LIKE $1 \
        AND ($2 IS NULL OR EXISTS ( \
            SELECT 1 \
            FROM subcategories as sc \
            JOIN categories AS c ON sc.parent_id = c.id \
            WHERE categories.id = sc.child_id AND c.title = $2 \
        ))
        ORDER BY title \
        LIMIT $3
        ",
        search,
        args.in_category,
        args.limit.unwrap_or(500)
    )
    .fetch_all(&pool)
    .await;

    match qry_result {
        Ok(v) if v.is_empty() => Ok(None),
        Ok(v) => Ok(Some(v)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(DestinyFetchError::SqlxErr(e)),
    }
}

pub fn construct_images_query(args: &SearchArgs) -> QueryBuilder<Sqlite> {
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM images ");

    q.push("WHERE LOWER(title) LIKE ");
    // let s = format!(r"'%{}", args.search.to_lowercase());
    q.push_bind(format!("'%{}%'", args.search.to_lowercase()));

    // filter for results who have args.in_category as a parent catgeory
    if let Some(c) = &args.in_category {
        q.push(
            r#" AND EXISTS (
            SELECT 1
            FROM image_categories as ic
            JOIN categories ON categories.id  = ic.category_id
            WHERE ic.image_id = images.id AND categories.title = "#,
        );
        q.push_bind(c);
        q.push("\n)");
    }

    // filter for images/files who have one of the specified extensions
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
            q.push_bind(min.timestamp());
            q.push(" AND ");
            q.push_bind(max.timestamp());
        }
        (Some(min), None) => {
            q.push("AND timestamp_ >= ");
            q.push_bind(min.timestamp());
        }
        (None, Some(max)) => {
            q.push("AND timestamp_ <= ");
            q.push_bind(max.timestamp());
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

    q.push(";");
    q
}

// MySQL version
// let ids = sqlx::query_as!(
//     Uuid,
//     "SELECT id FROM users \
//      WHERE (? IS NULL OR updated_at < ?) \
//        AND (? IS NULL OR updated_at > ?) \
//        AND (? IS NULL OR is_guest = ?)",
//     updated_before_option,
//     updated_before_option,
//     updated_after_option,
//     updated_after_option,
//     is_guest_option,
//     is_guest_option,
//     )
//     .fetch_all(&pool)
//     .await;

fn construct_categories_query(args: &SearchArgs) -> QueryBuilder<Sqlite> {
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM categories ");
    q.push("WHERE LOWER(title) LIKE ");
    q.push_bind(format!("'%{}%' ", args.search.to_lowercase()));
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
    use super::*;
    use futures::StreamExt;
    use sqlx::ConnectOptions;
    use sqlx::types::Text;
    use sqlx::{Connection, Execute, sqlite::SqliteConnectOptions};
    use std::{collections::HashSet, io::BufRead};
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
                titles: true,
                ids: false,
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

    fn load_exotic_results() -> HashSet<i64> {
        use std::fs;
        use std::io;
        use std::io::BufRead;
        let mut ids: HashSet<i64> = HashSet::new();
        let f = fs::File::open(Path::new("test-data/exotic_search_results.txt"))
            .expect("unable to open file");
        let lines = io::BufReader::new(f).lines();
        for line in lines {
            match line {
                Ok(s) => {
                    let (id_str, _) = s.split_once('|').expect("failed to parse file");
                    ids.insert(id_str.parse().unwrap());
                }
                Err(_) => (),
            }
        }
        ids
    }

    fn load_all_category_titles() -> HashSet<String> {
        use std::fs;
        use std::io;
        use std::io::BufRead;
        let f = fs::File::open(Path::new("test-data/all_category_titles.txt"))
            .expect("unable to open file");

        io::BufReader::new(f)
            .lines()
            .map_while(std::result::Result::ok)
            .collect()
    }

    #[tokio::test]
    async fn test_simple_sql1() {
        let args: SearchArgs = simple_image_search();
        let test: QueryBuilder<Sqlite> = construct_images_query(&args);
        let exp = "SELECT * FROM images WHERE LOWER(title) LIKE ? ORDER BY title";
        assert_eq!(test.into_string(), exp);
    }

    #[tokio::test]
    async fn test_static_search1() {
        let mut args = simple_image_search();
        args.search = "exotic".into();
        let pool = sqlx::SqlitePool::connect_with(
            SqliteConnectOptions::default().filename(Path::new("test-data/STATIC_TEST.db")),
        )
        .await
        .expect("unable to connect to database");
        let images = fetch_images(&args, pool)
            .await
            .expect("query failed")
            .expect("query returned 0 results");
        let exp_ids: HashSet<i64> = load_exotic_results();
        assert_eq!(images.len(), exp_ids.len());
        let test_ids: HashSet<i64> = images.into_iter().map(|i| i.id).collect();
        let diff = exp_ids.symmetric_difference(&test_ids).count();
        assert_eq!(diff, 0);
    }

    #[tokio::test]
    async fn test_static_search2() {
        let mut args = simple_image_search();
        args.search = "_".into();
        args.result_type.images = false;
        args.result_type.categories = true;
        let pool = sqlx::SqlitePool::connect("test-data/STATIC_TEST.db")
            .await
            .expect("unable to connect to database");
        let categories = fetch_categories(&args, pool)
            .await
            .expect("query failed")
            .expect("query returned no results");
        dbg!(categories);
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
