
use std::{fs::File};

use regex::{RegexBuilder};
use reqwest::blocking::Client;

use super::sitemap;

pub fn run() {
    meteor_download(); 
    //meteor_single_page("https://www.jotul.se/produkter/braskaminer/jotul-f-105-ll").unwrap();
}

fn meteor_download() {
    let pages = sitemap::get_pages("https://www.meteor.dk/sitemap.xml");
    
    for page in pages.iter().filter(|p| p.contains("com/product")) {
        meteor_single_page(page).unwrap();
    }
}

fn meteor_single_page(url :&str) -> Result<(), reqwest::Error> {
    println!("url: {}", url);

    let client = Client::new();
    let body = client.get(url).send()?.text()?;

    //<h1 class="product_title entry-title elementor-heading-title elementor-size-default">100CBS</h1>
    let model_regex = RegexBuilder::new("product_title(.*?)>(.*?)<")
        //.dot_matches_new_line(true)
        .build().unwrap();

    match model_regex.captures(&body){
        Some(capture) => {
            let model = capture.get(2).unwrap().as_str();
            
            let pdf_regex = RegexBuilder::new("<a href=\"([^\"]*?.pdf)\"(.*?)button-text\">([^<>\"]*?)<")
                .dot_matches_new_line(true)
                .build().unwrap();
            let caps = pdf_regex.captures_iter(&body);
            let pdfs :Vec<PdfLink>= caps.map(|c| PdfLink{
                brand: String::from("meteor"),
                model: String::from(model.trim()),
                filename:c.get(3).unwrap().as_str().to_string().replace(":", "_").replace("/", "_"),
                url: c.get(1).unwrap().as_str().to_string(),
            }).collect();
        
            for pdf in pdfs {
                download_pdf(&pdf);
            }
        
            Ok(())
        },
        None => {
            println!("No modelname found");
                Ok(())
        },
    }
}

fn download_pdf (pdf :&PdfLink) {
    //let url = fix_url(&pdf.url);
    let url = &pdf.url;
    let dir = format!("{}/{}", pdf.brand, pdf.model);
    let path = format!("{}/{}.pdf", dir, pdf.filename);

    if std::path::Path::new(path.as_str()).exists() {
        println!("File already exists: {}", path);
        return;
    }

    //thread::sleep(time::Duration::from_secs(2));
    println!("downloading: {:?}", pdf);
    println!("url: {:?}", &url);

    let client = Client::builder().danger_accept_invalid_certs(true).build().unwrap();
    let req = client.get(url)
        .header("Host", "webfiles.meteor.be")
        ;

    let mut content = req.send().expect(format!("failed to download: {}", url).as_str());

    std::fs::create_dir_all(&dir)
        .expect("failed to create dir");

    let mut file = File::create(path)
        .expect("Could not create file");

    std::io::copy(&mut content, &mut file)
        .expect("Could not write to file");

    println!("downloaded file");
}