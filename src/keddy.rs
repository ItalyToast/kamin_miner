use regex::{Regex};
use reqwest::blocking::Client;

use crate::download;

pub fn run() {
    keddy_download(); 
}

fn keddy_download() {
    println!("Keddy");
    let client = Client::new();
    let body = client.get("https://www.keddy.se/ladda_ner/").send().unwrap().text().unwrap();

    let pdf_regex = Regex::new("a href=[\"']([^<>]*?.pdf)[\"']>(.*?)<").unwrap();
    let caps = pdf_regex.captures_iter(&body);
    for c in caps {
        let result = download::pdf(download::PdfLink{
            brand    : "Keddy".to_string(),
            model    : "Keddy".to_string(),
            filename : c.get(2).unwrap().as_str().to_string(),
            url      : c.get(1).unwrap().as_str().to_string(),
        });

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }
}