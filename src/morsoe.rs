use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;

use crate::error::{Result, self};
use crate::download;

lazy_static!{
    static ref PRODUCT_REGEX : Regex = RegexBuilder::new("<div class=\"product-box\">(.*?)<strong class=\"product-title\">(.*?)<a href=\"(.*?)\"")
        .dot_matches_new_line(true)    
        .build().unwrap();

    static ref PDF_REGEX : Regex = RegexBuilder::new("href=\"(.*?)\".*?>(.*?)<")
        .build().unwrap();

    static ref REGION_REGEX : Regex = RegexBuilder::new("<div class=\"uk-h5 uk-text-uppercase uk-text-bold\">Diagrams, (.*?)</ul")
        .dot_matches_new_line(true)
        .build().unwrap();

    static ref MODEL_REGEX : Regex = regex::RegexBuilder::new("<meta property=\"og:title\" content=\"(.*?)\">")
        .build().unwrap();
}

pub fn run() {
    morsoe_download().unwrap();
}

fn morsoe_download() -> Result<()> {
    let pages = vec![
        "https://morsoe.com/en/product/indoor/wood-burning-stove",
        "https://morsoe.com/en/product/indoor/inserts",
    ];

    let client = Client::new();
    for page in pages {
        let body = client.get(page).send()?.text()?;

        for prodlink in PRODUCT_REGEX.captures_iter(&body) {
            let link = prodlink.get(3).unwrap().as_str();
            println!("{}", link);

            match morsoe_single_page(&client, &link) {
                Ok(_) => (),
                Err(e)=> println!("Failed to download url: {} {:?}", &link, e),
            };
        }
    }

    Ok(())
}

fn morsoe_single_page(client : &Client, url :&str) -> Result<()> {
    let body = client.get(url).send()?.text()?;
    let model = MODEL_REGEX
        .captures(&body).unwrap()
        .get(1).unwrap()
        .as_str()
        .replace("Morsø", "");

    println!("model {}", model);

    let document_region = REGION_REGEX
        .find(&body)
        .ok_or(error::message("Did not find document region"))
        ?.as_str();


    for doc in PDF_REGEX.captures_iter(document_region) {
        let result = download::pdf(download::PdfLink {
            brand    : "Morsoe".to_string(),
            model    : model.to_string(),
            url      : fix_url(doc.get(1).unwrap().as_str()),
            filename : doc.get(2).unwrap().as_str().to_string(),
        });

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }

    download::save_url("Morsoe".to_string(), model.to_string(), fix_url(url)).unwrap();
    Ok(())
}

fn fix_url(url: &str) -> String {
    if url.starts_with("https://morsoe.com/") {
        String::from(url)
    } else {
        format!("https://morsoe.com/{}", url)
    }
}