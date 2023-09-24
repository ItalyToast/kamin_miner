use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;

use crate::error::{Result, self};
use crate::download;

lazy_static!{
    static ref PDF_REGEX : Regex = RegexBuilder::new("<a class=\"button\" target=\"_blank\" href=\"([^\"]*?.pdf)\">(.*?)<")
        .dot_matches_new_line(true)
        .build().unwrap();

    static ref MODEL_REGEX : Regex = RegexBuilder::new("<h1 class=\"title\">(.*?)<")
        .build().unwrap();

    static ref LINK_REGEX : Regex = RegexBuilder::new("<a class=\"button\" href=\"(.*?)\"")
        .dot_matches_new_line(true)
        .build().unwrap();
}

pub fn run() {
    westbo_download().unwrap();
}

fn westbo_download() -> Result<()> {
    let pages = vec![
        "https://www.westbo.net/sv/produkter/",
    ];

    let client = Client::new();
    for page in pages {
        let body = client.get(page).send()?.text()?;

        for prodlink in LINK_REGEX.captures_iter(&body) {
            match westbo_single_page(&client, &fix_url(prodlink.get(1).unwrap().as_str())) {
                Ok(_) => (),
                Err(e)=> println!("Failed to download {:?}", e),
            };
        }
    }

    Ok(())
}


fn westbo_single_page(client :&Client, url :&str) -> Result<()> {
    println!("url {}", url);
    
    let body = client.get(url).send()?.text()?;
    let model = MODEL_REGEX
        .captures(&body).ok_or(error::message("no model found"))?
        .get(1).unwrap()
        .as_str();

    println!("model {}", model);

    let links: Vec<download::PdfLink> = PDF_REGEX
        .captures_iter(&body)
        .map(|l| {
            download::PdfLink {
                brand    : "Westbo".to_string(),
                model    : model.to_string(),
                url      : fix_url(l.get(1).unwrap().as_str()),
                filename : l.get(2).unwrap().as_str().to_string(),
            }
        })
        .collect();

    for link in links{
        let result = download::pdf(link);

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }

    download::save_url("Westbo".to_string(), model.to_string(), url.to_string()).unwrap();
    Ok(())
}

fn fix_url(url: &str) -> String {
    if url.starts_with("https://www.westbo.net") {
        String::from(url)
    } else {
        format!("https://www.westbo.net{}", url)
    }
}