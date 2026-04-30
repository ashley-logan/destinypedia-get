use serde_json::json;

pub fn page_images(titles: &[String]) -> serde_json::Value {
    let title: String = join_opts(titles);
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
            norm_titles.push(format!("File:{}", s));
        } else {
            norm_titles.push(s.clone());
        }
    }

    let title = join_opts(norm_titles.as_slice());

    json!({
        "action": "query",
        "titles": title,
        "prop": "imageinfo",
        "iiprop": "canonicaltitle|url|size",
        "format": "json"
    })
}

pub fn images(titles: &[String]) -> serde_json::Value {
    let title = join_opts(titles);

    json!({
        "action": "query",
        "titles": title,
        "prop": "images",
        "imlimit": "max",
        "format": "json"
    })
}

pub fn category_members(categories: &[String]) -> serde_json::Value {
    let mut ctitles: Vec<String> = vec![];
    // normalize page name for wiki api
    for s in categories {
        let ctitle: String = {
            if !s.starts_with("Category:") {
                format!("Category:{}", s)
            } else {
                s.clone()
            }
        };

        ctitles.push(ctitle);
    }

    let cmtitle: String = join_opts(ctitles.as_slice());

    json!({
        "action": "query",
        "list": "categorymembers",
        "cmtitle": cmtitle,
        "cmprop": "ids|title|type",
        "cmtype": "file",
        "cmlimit": "max",
        "format": "json"
    })
}

fn join_opts(opts: &[String]) -> String {
    opts.join(r"|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Result;
    use reqwest::blocking::{Client, Request};

    static BASE: &str = "https://www.destinypedia.com/api.php";

    fn get_client() -> Client {
        Client::builder()
            .user_agent("destiny-fetch-test")
            .build()
            .unwrap()
    }

    fn try_build_request<F>(s: &[String], func: F) -> (Client, Result<Request>)
    where
        F: Fn(&[String]) -> serde_json::Value,
    {
        let params = func(s);

        dbg!(&params);

        get_client().get(BASE).query(&params).build_split()
    }

    #[test]
    fn test_page_images_build() {
        let (_, r) =
            try_build_request(&["Dreaming City".into(), "Black_Fleet".into()], page_images);

        assert!(r.is_ok());
    }

    #[test]
    fn test_page_images_resp() {
        let (client, r) = try_build_request(&["Flower_game".into()], page_images);

        let resp = client.execute(r.unwrap());

        print!("{:?}", resp.unwrap());
    }

    #[test]
    fn test_image_info_build() {
        let (_, r) = try_build_request(
            &["Crota1.jpg".into(), "File:LoreUnviel.png".into()],
            image_info,
        );

        assert!(r.is_ok());
    }

    #[test]
    fn test_image_info_resp() {
        let (client, r) = try_build_request(&["Crota.jpg".into()], image_info);

        let resp = client.execute(r.unwrap());

        print!("{:?}", resp.unwrap());
    }

    #[test]
    fn test_images_build() {
        let (_, r) = try_build_request(&["Hive".into(), "Fallen".into()], images);

        assert!(r.is_ok());
    }

    #[test]
    fn test_images_resp() {
        let (client, r) = try_build_request(&["Crota's End".into()], images);

        let resp = client.execute(r.unwrap());

        print!("{:?}", resp.unwrap());
    }

    #[test]
    fn test_category_members_build() {
        let (_, r) = try_build_request(
            &["Expansions".into(), "Category:Raids".into()],
            category_members,
        );

        assert!(r.is_ok());
    }

    #[test]
    fn test_category_members_resp() {
        let (client, r) = try_build_request(&["Expansions".into()], category_members);

        let resp = client.execute(r.unwrap());

        print!("{:?}", resp.unwrap());
    }
}
