use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;

use crate::sitemap;
use crate::error::{Result, self};
use crate::download;

lazy_static!{
    static ref PDF_REGEX : Regex = RegexBuilder::new("a href=\"(.*?.pdf)\"(.*?)>(.*?)<")
        .build().unwrap();

    static ref MODEL_REGEX : Regex = RegexBuilder::new("product-single-title(.*?)<span>(.*?)<")
        .dot_matches_new_line(true)    
        .build().unwrap();
}

pub fn run() {
    scanspis_download(); 
}

fn scanspis_download() {
    let pages = sitemap::get_pages("https://www.scan-spis.se/sitemap.xml").unwrap();
    
    let client = Client::new();
    for page in pages.iter().filter(|p| p.contains("produkter")) {
        let result = scanspis_single_page(&client, page);
        if let Err(e) = result {
            println!("Failed to download page: {:?}", e);
        }
    }
}

fn scanspis_single_page(client :&Client, url :&str) -> Result<()> {
    println!("url: {}", url);

    let body = client.get(url).send()?.text()?;
    let model = MODEL_REGEX.captures(&body).ok_or(error::message("No model found"))?
        .get(2).unwrap()
        .as_str()
        .replace("SCAN", "")
        .trim()
        .to_string();
    
    for cap in PDF_REGEX.captures_iter(&body) {
        let result = download::pdf(download::PdfLink{
            brand    : "Scan-spis".to_string(),
            model    : model.to_string(),
            filename : cap.get(3).unwrap().as_str().to_string(),
            url      : cap.get(1).unwrap().as_str().to_string(),
        });

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }
    
    download::save_url("Scan-spis".to_string(), model.to_string(), url.to_string()).unwrap();
    Ok(())
}