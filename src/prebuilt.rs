use serde_json::json;

pub fn page_images(titles: &[String]) -> serde_json::Value {
    let title: String = titles.join("|");

    json!({
        "action": "query",
        "titles": title,
        "prop": "pageimages",
        "piprop": "name|original",
        "format": "json"
    })
}

pub fn image_info(titles: &[String]) -> serde_json::Value {
    let mut norm_titles: Vec<String> = vec![];
    // normalize image titles to format preferred by the media wiki api
    for s in titles {
        if !s.starts_with("File:") {
            norm_titles.push("File:".to_string() + s);
        } else {
            norm_titles.push(s.clone());
        }
    }

    let title = norm_titles.join("|");

    json!({
        "action": "query",
        "titles": title,
        "prop": "imageinfo",
        "iiprop": "canonicaltitle|url|size",
        "format": "json"
    })
}

pub fn images(titles: &[String]) -> serde_json::Value {
    let title = titles.join("|");

    json!({
        "action": "query",
        "titles": title,
        "prop": "images",
        "imlimit": "max",
        "format": "json"
    })
}

pub fn category_members(category: String) -> serde_json::Value {
    // normalize page name for wiki api
    let ctitle: String = {
        if !category.starts_with("Category:") {
            format!("Category:{}", category)
        } else {
            category
        }
    };

    json!({
        "action": "query",
        "list": "categorymembers",
        "cmtitle": ctitle,
        "cmprop": "ids|title|type",
        "cmtype": "file",
        "cmlimit": "max"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client;

    static BASE: &str = "https://destinypedia/api.php";

    #[test]
    fn test_page_images_build() {
        let params = page_images(&vec!["Dreaming City".into(), "Black_Fleet".into()]);

        dbg!(&params);

        let r = Client::new().get(BASE).query(&params).build().unwrap();

        dbg!("URLLLLLLLL {}", r.url());

        assert!(false);
    }
}
