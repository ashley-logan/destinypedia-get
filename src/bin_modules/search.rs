use crate::bin_modules::cli::{DetailLevel, FileType, ResultType, SearchArgs};
use crate::bin_modules::{Categories, ImageCategories, Images, Subcategories};
use crate::bin_modules::{categories, image_categories, images, subcategories};
use crate::{DestinyFetchError, Result};
use csv::Writer;
use diesel::prelude::*;
use std::ops::Mul;
use std::path::{Path, PathBuf};

pub fn search(args: SearchArgs) -> Result<()> {
    let mut conn = SqliteConnection::establish("data/dev.db")?;
    let (imgs, cats) = match args.result_type {
        ResultType { all: true, .. } => (
            search_images(&args, &mut conn)?,
            search_categories(&args, &mut conn)?,
        ),
        ResultType { images: true, .. } => (search_images(&args, &mut conn)?, vec![]),
        ResultType {
            categories: true, ..
        } => (vec![], search_categories(&args, &mut conn)?),
        _ => return Err(DestinyFetchError::MissingArgErr),
    };
    match args.output {
        Some(f) => {
            output_search(f.as_path(), imgs, cats)?;
        }
        None => (),
    }
    Ok(())
}

fn output_search(file: &Path, imgs: Vec<Images>, cats: Vec<Categories>) -> Result<()> {
    let mut w = csv::Writer::from_path(file)?;

    for img in imgs {
        w.serialize(img)?;
    }
    w.flush()?;

    for cat in cats {
        w.serialize(cat)?;
    }

    Ok(())
}

pub fn search_images(args: &SearchArgs, conn: &mut SqliteConnection) -> Result<Vec<Images>> {
    match &args.result_type {
        ResultType {
            categories: true, ..
        } => return Err(DestinyFetchError::WrongQueryMethod),
        _ => (),
    };
    let mut q: images::BoxedQuery<'_, diesel::sqlite::Sqlite> = images::table.into_boxed();

    // filter by contains <SEARCH> as a substring
    q = q.filter(images::title.like(format!("%{}%", &args.search)));

    if let Some(v) = &args.ftype {
        let mut s: std::vec::IntoIter<String> = as_string_vec(v).into_iter();

        let pat = format!("%{}", s.next().unwrap_or_default());
        q = q.filter(images::title.like(pat));

        for ext in s {
            let n_pat = format!("%{}", &ext);
            q = q.or_filter(images::title.like(n_pat));
        }
    }

    match &args.maxsize {
        Some(i) if *i >= 0 => {
            q = q.filter(images::size.le(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.minsize {
        Some(i) if *i >= 0 => {
            q = q.filter(images::size.ge(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.maxwidth {
        Some(i) if *i >= 0 => {
            q = q.filter(images::width.le(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.minwidth {
        Some(i) if *i >= 0 => {
            q = q.filter(images::width.ge(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.maxheight {
        Some(i) if *i >= 0 => {
            q = q.filter(images::height.le(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.minheight {
        Some(i) if *i >= 0 => {
            q = q.filter(images::height.ge(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.maxpixels {
        Some(i) if *i >= 0 => {
            q = q.filter(images::width.mul(images::height).le(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.minpixels {
        Some(i) if *i >= 0 => {
            q = q.filter(images::width.mul(images::height).ge(*i));
        }
        Some(n) => {
            return Err(DestinyFetchError::NegativeArgErr);
        }
        _ => (),
    };

    match &args.before {
        Some(dt) => {
            q = q.filter(images::timestamp_.lt(dt.naive_utc()));
        }
        _ => (),
    };

    match &args.after {
        Some(dt) => {
            q = q.filter(images::timestamp_.gt(dt.naive_utc()));
        }
        _ => (),
    };

    let results: Vec<Images> = match &args.in_category {
        Some(cat) => {
            let c: &str = match cat.strip_prefix("Category:") {
                Some(s) => s,
                None => cat.as_str(),
            };
            let id_: i32 = categories::table
                .filter(categories::title.eq(c))
                .select(categories::id)
                .first(conn)?;
            q.inner_join(image_categories::table)
                .filter(image_categories::category_id.eq(id_))
                .select(Images::as_select())
                .order_by(images::title)
                .limit(args.limit.clone().unwrap_or(1_000_000).into())
                .load(conn)?
        }
        None => q
            .select(Images::as_select())
            .order_by(images::title)
            .limit(args.limit.clone().unwrap_or(1_000_000).into())
            .load(conn)?,
    };

    Ok(results)
}

pub fn search_categories(
    args: &SearchArgs,
    conn: &mut SqliteConnection,
) -> Result<Vec<Categories>> {
    match &args.result_type {
        ResultType { images: true, .. } => return Err(DestinyFetchError::WrongQueryMethod),
        _ => (),
    };
    let mut q: categories::BoxedQuery<'_, diesel::sqlite::Sqlite> = categories::table.into_boxed();

    q = q.filter(categories::title.like(format!("%{}%", &args.search)));

    let results: Vec<Categories> = match &args.in_category {
        Some(cat) => {
            let c: &str = match cat.strip_prefix("Category:") {
                Some(s) => s,
                None => cat.as_str(),
            };
            let id_: i32 = categories::table
                .filter(categories::title.eq(cat))
                .select(categories::id)
                .first(conn)?;
            q.inner_join(subcategories::table.on(categories::id.eq(subcategories::subcategory_id)))
                .filter(subcategories::category_id.eq(id_))
                .select(Categories::as_select())
                .order_by(categories::title)
                .limit(args.limit.clone().unwrap_or(1_000_000).into())
                .load(conn)?
        }
        None => q
            .select(Categories::as_select())
            .order_by(categories::title)
            .limit(args.limit.clone().unwrap_or(1_000_000).into())
            .load(conn)?,
    };

    Ok(results)
}

fn as_string_vec(ftypes: &Vec<FileType>) -> Vec<String> {
    let mut v: Vec<String> = vec![];

    for f in ftypes {
        match f {
            FileType::PNG => v.push(".png".into()),
            FileType::JPG => v.extend_from_slice(&[".jpg".into(), ".jpeg".into()]),
            FileType::HEIC => v.push(".heic".into()),
            FileType::WEBP => v.push(".webp".into()),
            FileType::GIF => v.push(".gif".into()),
            FileType::SVG => v.push(".svg".into()),
        };
    }

    v
}
