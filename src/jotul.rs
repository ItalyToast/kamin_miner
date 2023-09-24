use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;

use crate::download;
use crate::error::{Result, self};
use super::sitemap;

lazy_static!{
    static ref MODEL_REGEX : Regex = RegexBuilder::new("product-single-title(.*?)<span>(.*?)<")
        .dot_matches_new_line(true)
        .build().unwrap();

    //<a href="https://www.jotul.se/sites/sweden/files/products/Manual_F_105_R_10051250_P06_121219_SE_FI.pdf" type="application/pdf; length=2899495" target="_blank">Bruksanvisning</a>
    static ref PDF_REGEX : Regex = RegexBuilder::new(r#"a href="(.*?.pdf)"(.*?)>(.*?)<"#)
        .build().unwrap();
}

pub fn run() {
    jotul_download(); 
}

fn jotul_download() {
    let client = Client::new();
    let pages = sitemap::get_pages("https://www.jotul.se/sitemap.xml").unwrap();

    for page in pages.iter().filter(|p| p.contains("produkter")) {
        let result = jotul_single_page(&client, page);
        if let Err(e) = result {
            println!("Failed to download page: {:?}", e);
        }
    }
}

fn jotul_single_page(client : &Client, url :&str) -> Result<()> {
    println!("url: {}", url);

    let body = client.get(url).send()?.text()?;
    let model = MODEL_REGEX
        .captures(&body).ok_or(error::message("No model found"))?
        .get(2).unwrap()
        .as_str()
        .to_uppercase()
        .replace("JØTUL", "")
        .trim()
        .to_string();

    for c in PDF_REGEX.captures_iter(&body) {
        let result = download::pdf(download::PdfLink{
            brand    : "Jotul".to_string(),
            model    : model.clone(),
            filename : c.get(3).unwrap().as_str().to_string(),
            url      : c.get(1).unwrap().as_str().to_string(),
        });

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }

    download::save_url("Jotul".to_string(), model.trim().to_string(), url.to_string()).unwrap();
    Ok(())
}