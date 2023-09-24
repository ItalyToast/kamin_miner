use std::collections::HashSet;
use std::fs::File;
use std::{thread, time};

use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;

use crate::error::{Result, self};
use crate::download;

lazy_static!{
    static ref PDF_REGEX : Regex = RegexBuilder::new(r#"<li class="elementor-icon-list-item">\s*<a\s*href="([^"]*?)"(.*?)<span class="elementor-icon-list-text">(.*?)<"#)
        .dot_matches_new_line(true)
        .build().unwrap();

    static ref MODEL_REGEX : Regex = RegexBuilder::new("<h2 class=\"elementor-heading-title elementor-size-default\">(.*?)<")
        .build().unwrap();

    static ref LINK_REGEX : Regex = RegexBuilder::new("<a class=\"jet-button__instance jet-button__instance--icon-right hover-effect-1\" href=\"([^\"]*?)\">")
        .dot_matches_new_line(true)
        .build().unwrap();

    static ref SKIP_URLS : HashSet<String> = HashSet::from([
            "https://termatech.com/sv/hitta-aterforsaljare/".to_string(),
            ]);
}

pub fn run() {
    termatech_download().unwrap();
}

fn termatech_download() -> Result<()> {
    let pages = vec![
        "https://termatech.com/sv/serier/tt20-serien",
        "https://termatech.com/sv/serier/tt21-serien",
        "https://termatech.com/sv/serier/tt22-serien",
        "https://termatech.com/sv/serier/tt23-serien",
        "https://termatech.com/sv/serier/tt30-serien",
        "https://termatech.com/sv/serier/tt60-serien",
        "https://termatech.com/sv/serier/tt80-serien",
    ];

    let client = Client::new();
    for page in pages {
        let body = client.get(page).send()?.text()?;

        for prodlink in LINK_REGEX.captures_iter(&body) {
            let link = fix_url(prodlink.get(1).unwrap().as_str());
            println!("{}", link);

            if SKIP_URLS.contains(&link) {
                println!("Skipping...");
                continue;
            }

            match termatech_single_page(&client, &link) {
                Ok(_) => (),
                Err(e)=> println!("Failed to download url: {} {:?}", &link, e),
            };
        }
    }

    Ok(())
}


fn termatech_single_page(client :&Client, url :&str) -> Result<()> {
    rate_limit();

    let body = client.get(url).send()?.text()?;
    let model = MODEL_REGEX
        .captures(&body).ok_or(error::message("no model found"))?
        .get(1).unwrap()
        .as_str();

    println!("model {}", model);

    //<li class="elementor-icon-list-item"> <a href="https://termatech.com/wp-content/uploads/dealerdata/br%C3%A6ndeovne/tt20/tt20r/se/dop.pdf" target="_blank"> <span class="elementor-icon-list-icon"> <i aria-hidden="true" class="fas fa-long-arrow-alt-right"></i> </span> <span class="elementor-icon-list-text">DOP / Prestandadeklartion</span> </a> </li>

    let links: Vec<download::PdfLink> = PDF_REGEX
        .captures_iter(&body)
        .map(|l| {
            download::PdfLink {
                brand    : "TermaTech".to_string(),
                model    : model.to_string(),
                url      : fix_url(l.get(1).unwrap().as_str()),
                filename : l.get(3).unwrap().as_str().to_string(),
            }
        })
        .filter(|pdf| !pdf.filename.contains("katalog") && !pdf.filename.contains("prislista"))
        .collect();

    println!("{} pdfs found.", links.len());

    if links.len() == 0 {
        println!("{}", &body);
    }

    for link in links.iter().filter(|l| l.url.ends_with(".pdf")){
        download_pdf(link);
    }

    download::save_url("TermaTech".to_string(), model.to_string(), url.to_string()).unwrap();
    Ok(())
}

fn download_pdf (pdf :&download::PdfLink) {
    let pdf = download::PdfLink{
        brand    : download::fix_filename(pdf.brand.clone()),
        model    : download::fix_filename(pdf.model.clone()),
        filename : download::fix_filename(download::ensure_extension(&pdf.filename, ".pdf")),
        url      : pdf.url.clone(),
    };

    let url = fix_url(&pdf.url);
    let dir = format!("{}/{}", pdf.brand, pdf.model);
    let path = format!("{}/{}", dir, pdf.filename);

    if std::path::Path::new(path.as_str()).exists() {
        println!("File already exists: {}", path);
        return;
    }

    rate_limit();
    println!("downloading: {:?}", pdf);
    println!("url: {:?}", &url);

    let client = Client::new();
    let req = client.get(&url)
        .header("Host", "termatech.com");

    let mut content = req.send().expect(format!("failed to download: {}", url).as_str());

    std::fs::create_dir_all(&dir)
        .expect("failed to create dir");

    let mut file = File::create(path)
        .expect("Could not create file");

    std::io::copy(&mut content, &mut file)
        .expect("Could not write to file");

    println!("downloaded file");
}

fn rate_limit() {
    println!("rate limited...");
    thread::sleep(time::Duration::from_secs(1));
}

fn fix_url(url: &str) -> String {
    if url.starts_with("https://termatech.com/") {
        String::from(url)
    } else {
        format!("https://termatech.com/{}", url)
    }
}