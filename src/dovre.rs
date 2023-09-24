use std::collections::HashSet;
use std::fs;

use regex::{RegexBuilder, Regex};
use reqwest::blocking::Client;

use crate::download;
use crate::sitemap;
use crate::error::{Result, self};

lazy_static!{
    //<h1 class="product_title entry-title elementor-heading-title elementor-size-default">100CBS</h1>
    static ref MODEL_REGEX : Regex = RegexBuilder::new("product_title(?:.*?)>(.*?)<")
        .build().unwrap();

    static ref PDF_REGEX : Regex = RegexBuilder::new("<a href=\"([^\"]*?.pdf)\"(?:.*?)button-text\">([^<>\"]*?)<")
        .dot_matches_new_line(true)
        .build().unwrap();
}


pub fn run() {
    dovre_download(); 
}

fn dovre_download() {
    let client = Client::new();
    let pages = sitemap::get_pages("https://dovrefire.com/sitemap_index.xml").unwrap();
    let skips = skip_urls();

    for page in pages.iter().filter(|p| p.contains("com/product") && !skips.contains(*p)) {
        let result = dovre_single_page(&client, page);
    
        if let Err(e) = result {
            println!("Failed page : {} {:?}", page, e);
        }
    }
}

fn dovre_single_page(client : &Client, url :&str) -> Result<()> {
    println!("url: {}", url);

    let body = client.get(url).send()?.text()?;

    let model = MODEL_REGEX.captures(&body).ok_or(error::message("No model found"))?
        .get(1).unwrap()
        .as_str()
        .trim();

        
        for cap in PDF_REGEX.captures_iter(&body) {
            let result = download::pdf(download::PdfLink{
                brand    : "Dovre".to_string(),
                model    : model.to_string(),
                filename : cap.get(2).unwrap().as_str().to_string(),
            url      : cap.get(1).unwrap().as_str().to_string(),
        });
        
        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }
    
    download::save_url("Dovre".to_string(), model.to_string(), url.to_string()).unwrap();
    Ok(())
}


fn skip_urls() -> HashSet<String> {
    fs::read_to_string("dovre_skip.txt") 
        .unwrap()  // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()
}