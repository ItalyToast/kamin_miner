
use std::{fs::File};

use regex::{RegexBuilder, Regex};
use reqwest::blocking::Client;

use crate::download::PdfLink;

lazy_static!{
    //<h1 class="product_title entry-title elementor-heading-title elementor-size-default">100CBS</h1>
    static ref PDF_REGEX : Regex = RegexBuilder::new(r#"a href="(.*?.pdf)">(.*?)</a>"#)
        .build().unwrap();
}

pub fn run() {
    shiedel_download(); 
}

fn shiedel_download() {
    let client = Client::new();
    //<a href="https://www.schiedel.com/se/wp-content/uploads/sites/20/2022/09/bestallning-offertunderlag-aug2022.pdf">Offert &amp; Beställning – Permeter</a>
    let body = client.get("https://www.schiedel.com/se/ladda-hem/").send().unwrap().text().unwrap();

    let caps = PDF_REGEX.captures_iter(&body);

    let pdfs :Vec<PdfLink>= caps.map(|c| PdfLink{
        brand: String::from("Shiedel"),
        model: String::from("Shiedel"),
        filename:c.get(2).unwrap().as_str().to_string().replace(":", "_").replace("/", "_"),
        url: c.get(1).unwrap().as_str().to_string(),
    }).collect();

    for pdf in pdfs {
        if let Err(ex) = crate::download::pdf(pdf) {
            println!("{:?}", ex);
        }
    }
}