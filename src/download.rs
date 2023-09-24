use std::{fs::File, io::Write};

use regex::Regex;
use reqwest::{blocking::{Client, ClientBuilder}};

use crate::error;

const PDF_SIG: [u8;5] = [0x25, 0x50, 0x44, 0x46, 0x2D];

const URL_TEMPLATE : &str = r#"<!DOCTYPE html>
<html>
  <head>
    <meta http-equiv="refresh" content="1; url='{{URL}}'" />
  </head>
  <body>
    <p>You will be redirected to {{URL}} soon!</p>
  </body>
</html>"#;

lazy_static! {
    static ref HOST_REGEX: Regex = Regex::new("://(.*?)/").unwrap();

    static ref CLIENT: Client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
}

#[derive(Debug)]
pub struct PdfLink {
    pub brand :String,
    pub model :String,
    pub filename :String,
    pub url :String,
}

pub fn save_url(brand : String, model : String, url : String) -> crate::Result<()>{
    let brand = fix_filename(brand);
    let model = fix_filename(model);

    let dir = format!("{}/{}", brand, model);
    let path = format!("{}/{}/{}", brand, model, "url.html");

    if !std::path::Path::new(&dir).exists() {
        return Ok(());
    }

    let mut f = File::create(path)?;

    let content = URL_TEMPLATE.to_string()
        .replace("{{URL}}", &url);


    if let Err(_) = f.write(content.as_bytes()) {
        println!("Failed to write URL file");
    }
    Ok(())
}

pub fn pdf (pdf :PdfLink) -> crate::Result<()>{
    let pdf = PdfLink{
        brand    : fix_filename(pdf.brand),
        model    : fix_filename(pdf.model),
        filename : fix_filename(ensure_extension(&pdf.filename, ".pdf")),
        url      : pdf.url,
    };

    let brand_letter = pdf.brand.chars().nth(0).unwrap().to_ascii_uppercase();
    let dir = format!("{}/{}", pdf.brand, pdf.model);
    let mut path = format!("{}/{}/{}", pdf.brand, pdf.model, pdf.filename);
    let insert_pos = path.chars().position(|c| c == '.').unwrap();
    path.insert_str(insert_pos, &format!(" [{} {}]", brand_letter, pdf.model));
    
    let host = match HOST_REGEX.captures(&pdf.url) {
        Some(cap) => cap.get(1).unwrap().as_str(),
        None => return Err(error::message(&format!("no host found: {}", pdf.url))),
    };

    if std::path::Path::new(&path).exists() {
        println!("File already exists: {}", &path);
        return Err(error::message(&format!("File already exists: {}", &path)));
    }

    println!("downloading: {:?}", pdf);
    println!("url: {:?}", pdf.url);

    let req = CLIENT.get(&pdf.url)
        .header("Host", host)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36");

    let content = req.send()?;
    let status = content.status();
    if !status.is_success() {
        return Err(error::MinerError::Message(format!("Server error: {}", status.as_u16()).to_string()));
    }

    let bytes = content.bytes();
    match bytes {
        Err(ex) => return Err(error::MinerError::Reqwest(ex)),
        Ok(data) => {
            if data.len() > 5 && data[0..5] == PDF_SIG {
                //Copy file to disk
                std::fs::create_dir_all(&dir)?;
                let mut file = File::create(path)?;
                match file.write_all(&data) {
                    Ok(_) =>  println!("downloaded file of size: {}", data.len()),
                    Err(_) => println!("error downloading file {}", pdf.url), 
                }
                Ok(())
            } else
            {
                Err(error::MinerError::Message("Invalid PDF".to_string()))
            }
        },
    }
}

pub fn fix_filename(filename :String) -> String {
    filename
        .replace("/", ",")
        .replace(":", "_")
        .replace("å", "a")
        .replace("ä", "a")
        .replace("ö", "o")
        .replace("ø", "o")
        .replace("Å", "A")
        .replace("Ä", "A")
        .replace("Ö", "O")
        .replace("Ö", "O")
        .replace("Ø", "O")
        .replace("\u{B1}", "^1")
        .replace("\u{B2}", "^2")
        .replace("\u{B3}", "^3")
        .replace("\u{B9}", "1")
        .replace("\u{BA}", "2")
        .replace("\u{BB}", "3")
        .trim()
        .to_string()
}

pub fn ensure_extension(str :&str, ext :&str) ->String {
    if !str.ends_with(ext){
        str.to_string() + ext
    } else {
        str.to_string()
    }
}