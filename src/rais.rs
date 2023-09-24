use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;

use crate::error::{Result, self};
use crate::download::{self};

lazy_static!{
    static ref ISSU_PDF_REGEX : Regex = RegexBuilder::new(r#"a href="(https://issuu.com/.*?)"(.*?)>(.*?)<"#)
        .build().unwrap();

    static ref ISSU_DATA_JSON_REGEX : Regex = RegexBuilder::new(r#"data-json="(.+?)""#)
        .build().unwrap();

    static ref ISSU_ID_REGEX : Regex = RegexBuilder::new(r#"<meta content="https://image\.isu\.pub/(.+?)-(.+?)/jpg/page_1\.jpg"#)
        .build().unwrap();

    static ref PDF_REGEX : Regex = RegexBuilder::new(r#"a href="(.*?.pdf)"(.*?)>(.*?)<"#)
        .build().unwrap();

    static ref MODEL_REGEX : Regex = RegexBuilder::new(r#"h1 class="uk-margin">(.*?)</h1"#)
        .dot_matches_new_line(true)    
        .build().unwrap();

    static ref LINK_REGEX : Regex = RegexBuilder::new(r#"href="(.*?)""#)
        .build().unwrap();
}

pub fn run() {
    match rais_download() {
        Ok(_) => println!("Rais complete"),
        Err(e) => println!("Rais failed: {}", e),
    }; 
}

fn rais_download() -> Result<()> {
    let pages : Vec<&str> = vec![
        "https://www.rais.com/se/insatser",
        "https://www.rais.com/se/braskaminer",
        //"https://www.rais.com/se/gaskamin/inbyggda-gaskaminer",
        //"https://www.rais.com/se/gaskamin/fristaende-gaskaminer",
    ];

    for page in pages {
        println!("{}", page);

        let client = Client::new();
        let body = client.get(page)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36")
            .send()?.text()?;

        let links :Vec<&str>= LINK_REGEX
            .captures_iter(&body)
            .map(|l| l.get(1).unwrap().as_str())
            .filter(|l| l.contains("/se/insatser/") || l.contains("/se/braskaminer/") || l.contains("/se/gaskamin"))
            .collect();

        for link in links {
            match rais_single_page(&client, &fix_url(link)){
                Ok(_) => (),
                Err(e) => println!("failed to download product: {}", e),
            }
        }
    }
    Ok(())
}

fn rais_single_page(client :&Client, url :&str) -> Result<()> {
    println!("url: {}", url);

    let body = client
        .get(url)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36")
        .send()?
        .text()?;

    let model = MODEL_REGEX
        .captures(&body).ok_or(error::message("No model found"))?
        .get(1).unwrap()
        .as_str()
        .replace("RAIS", "")
        .trim()
        .to_string();

    //<a href="https://www.rais.se/sites/sweden/files/products/Manual_F_105_R_10051250_P06_121219_SE_FI.pdf" type="application/pdf; length=2899495" target="_blank">Bruksanvisning</a>
    let caps = PDF_REGEX.captures_iter(&body);
    let mut pdfs :Vec<download::PdfLink>= caps.map(|c| download::PdfLink{
        brand    : "Rais".to_string(),
        model    : model.to_string(),
        filename : c.get(3).unwrap().as_str().to_string(),
        url      : fix_url(c.get(1).unwrap().as_str()),
    }).collect();

    pdfs.append(&mut get_issu_pdf_links(&model, &body));

    for pdf in pdfs {
        let result = download::pdf(pdf);

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }

    download::save_url("Rais".to_string(), model.to_string(), url.to_string()).unwrap();
    Ok(())
}

fn get_issu_pdf_links(model: &str, body : &str) -> Vec<download::PdfLink> {
    let client = Client::new();
    let mut result : Vec<download::PdfLink> = vec![];

    for link in ISSU_PDF_REGEX.captures_iter(&body) {
        let url = link.get(1).unwrap().as_str().to_string();
        let name = link.get(3).unwrap().as_str().to_string();

        let body = client.get(&url)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36")
            .send().unwrap().text().unwrap();

        let cap = match ISSU_ID_REGEX.captures(&body) {
            Some(cap) => cap,
            None => {
                println!("No ID found");
                continue;
            },
        };

        let id = cap
            .get(2)
            .unwrap()
            .as_str()
            .to_string();

        println!("{}", id);

        let body2 = client.get(format!("https://issuu.com/call/backend-reader3/download/{}", id))
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36")
            .send().unwrap()
            .text().unwrap();

        let json :serde_json::Value= serde_json::from_str(&body2).unwrap();
        let url_pdf = json.get("url").unwrap()
            .as_str().unwrap()
            .to_string();
        
        result.push(download::PdfLink{
            brand: "Rais".to_string(),
            model: model.to_string(),
            filename: name,
            url: url_pdf,
        });
    }
    result 
}

fn fix_url(url: &str) -> String {
    if url.starts_with("https://www.rais.com") {
        String::from(url)
    } else {
        format!("https://www.rais.com{}", url)
    }
}