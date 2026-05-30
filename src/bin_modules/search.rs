use crate::bin_modules::cli::{DetailLevel, ResultType, SearchArgs};

use crate::bin_modules::database::rows::ImagesRow;
use crate::{DestinyFetchError, Result};
use csv::Writer;
use futures::TryStreamExt;
use sqlx::SqlitePool;
use sqlx::query::QueryAs;
use sqlx::{QueryBuilder, sqlite::Sqlite};
use std::ops::Mul;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::io::{self, AsyncWriteExt};

/*
pub fn fetch<'e, 'c, E>(
    self,
    executor: E,
) -> Pin<Box<dyn Stream<Item = Result<O, Error>> + Send + 'e>>
where
    'c: 'e,
    'q: 'e,
    E: 'e + Executor<'c, Database = DB>,
    DB: 'e,
    O: 'e,
    A: 'e,
*/

pub type ImagesQuery<'a> = QueryAs<'a, Sqlite, ImagesRow, sqlx::sqlite::SqliteArguments>;

pub fn construct_images_query(args: &SearchArgs) -> QueryBuilder<Sqlite> {
    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM images ");

    q.push("WHERE title LIKE ");
    q.push_bind(format!("%{}% ", &args.search));

    if let Some(c) = &args.in_category {
        q.push(
            r#"AND EXISTS (
            SELECT 1
            FROM image_categories as ic
            JOIN categories ON categories.id  = ic.category_id
            WHERE ic.image_id = images.id AND categories.title = ?
        ) "#,
        );
        q.push_bind(c);
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
            q.push("AND size_ BETWEEN ? AND ? ");
            q.push_bind(min);
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND size_ >= ? ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND size_ <= ? ");
            q.push_bind(max);
        }
        _ => (),
    }

    match (&args.after, &args.before) {
        (Some(min), Some(max)) => {
            q.push("AND timestamp_ BETWEEN ? AND ? ");
            q.push_bind(min.timestamp());
            q.push_bind(max.timestamp());
        }
        (Some(min), None) => {
            q.push("AND timestamp_ >= ? ");
            q.push_bind(min.timestamp());
        }
        (None, Some(max)) => {
            q.push("AND timestamp_ <= ? ");
            q.push_bind(max.timestamp());
        }
        _ => (),
    }

    match (&args.minwidth, &args.maxwidth) {
        (Some(min), Some(max)) => {
            q.push("AND width BETWEEN ? AND ? ");
            q.push_bind(min);
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND width >= ? ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND width <= ? ");
            q.push_bind(max);
        }
        _ => (),
    }
    match (&args.minheight, &args.maxheight) {
        (Some(min), Some(max)) => {
            q.push("AND height BETWEEN ? AND ? ");
            q.push_bind(min);
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND height >= ? ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND height <= ? ");
            q.push_bind(max);
        }
        _ => (),
    }

    match (&args.minpixels, &args.maxpixels) {
        (Some(min), Some(max)) => {
            q.push("AND width * height BETWEEN ? AND ? ");
            q.push_bind(min);
            q.push_bind(max);
        }
        (Some(min), None) => {
            q.push("AND width * height >= ? ");
            q.push_bind(min);
        }
        (None, Some(max)) => {
            q.push("AND width * height <= ? ");
            q.push_bind(max);
        }
        _ => (),
    }

    q.push("ORDER BY title");

    match &args.limit {
        Some(lim) if *lim >= 0_i32 => {
            q.push(" LIMIT ?");
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

pub fn search(args: SearchArgs, conn: &SqlitePool) -> Result<()> {
    let (mut images_q, mut cats_q): (Option<QueryBuilder<Sqlite>>, Option<QueryBuilder<Sqlite>>) =
        (None, None);

    if let ResultType {
        categories: true, ..
    }
    | ResultType { all: true, .. } = &args.result_type
    {
        cats_q = Some(construct_categories_query(&args));
    }
    if let ResultType { images: true, .. } | ResultType { all: true, .. } = &args.result_type {
        images_q = Some(construct_categories_query(&args));
    }
    if let Some(mut q) = images_q {
        let img_stream = q.build_query_as::<'_, ImagesRow>().fetch(conn);
    }

    if let Some(mut q) = cats_q {
        let cat_stream = q.build_query_as::<'_, ImagesRow>().fetch(conn);
    }

    Ok(())
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
