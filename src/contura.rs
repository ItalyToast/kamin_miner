

use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};

use crate::error::{Result, self};
use crate::download;

lazy_static!{

    static ref MODEL_REGEX : Regex = RegexBuilder::new(r#"<h1 class="ProductIntro-productTitle">(.+?)</h1>"#)
        .build().unwrap();

    static ref V2_REGEX : Regex = RegexBuilder::new(r#"<a shape="rect" href="(.+?)"(.+?)<a>(.+?)</a></dt>"#)
        .build().unwrap();

    static ref JSON_REGEX : Regex = RegexBuilder::new("<script>var product = (?P<json>.*?)</")
        .build().unwrap();

    static ref PAGE_REGEX : Regex = regex::RegexBuilder::new("ProductListing-productName name\">\\s*?<a href=\"(.*?)\">\\s*?(.*?)</a>")
        .build().unwrap();
}

pub fn run() {
    contura_download().unwrap();
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct ConturaKaminJSON {
    systemId :i64,
    name :String,
    language :String,
    market :String,
    brand :String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct ConturaApiDocumentJSON {
    description :String,
    documentUrl :String,
}

fn contura_download() -> Result<()> {
    let pages = vec![
        "https://www.contura.eu/sv-se/kaminer/braskaminer",
        "https://www.contura.eu/sv-se/kaminer/taljstenskaminer",
        "https://www.contura.eu/sv-se/kaminer/eldstader-och-murspisar",
        "https://www.contura.eu/sv-se/kaminer/kassetter--insatser",
        "https://www.contura.eu/sv-se/kaminer/gjutjarnskaminer",
    ];

    let client = Client::new();
    for page in pages {
        let body = client.get(page).send()?.text()?;

        let caps = PAGE_REGEX.captures_iter(&body);
        for prod in caps{
            let url =  fix_url(prod.get(1).unwrap().as_str());
            println!("url {}", &url);
            match contura_single_page_v2(&client, &url) {
                Ok(_) => (),
                Err(e)=> println!("Failed to download url: {} {:?}", &url, e),
            };
        }
    }

    Ok(())
}

fn contura_single_page_v1(client :&Client, url :&str) -> Result<()> {
    let body = client.get(url).send()?.text()?;

    let caps = JSON_REGEX.captures(&body);
    let json = match caps {
        Some(j) => j.name("json").unwrap(),
        None => return Err(error::message("did not find react initalstate json")),
    };

    let val:serde_json::Value = serde_json::from_str(json.as_str())?;
    let p :ConturaKaminJSON= val.get("product").unwrap().to_owned().into();

    println!("p = {:?}", p);
    
    let links = contura_get_pdf_links(client, p.systemId.to_string().as_str(), &p.language, &p.market, &p.brand, &p.name);
    println!("json = {:?}", links);
    match links {
        Ok(links) => {
            for link in links{
                if let Err(e) = download::pdf(link) {
                    println!("Failed to download pdf: {:?}", e);
                }
            }
        },
        Err(e) => println!("Failed to aquire pdf links: {:?}", e),
    }
    
    download::save_url("Contura".to_string(), p.name.to_string(), url.to_string()).unwrap();
    Ok(())
}

fn contura_single_page_v2(client :&Client, url :&str) -> Result<()> {
    let body = client.get(url).send()?.text()?;

    let model = MODEL_REGEX
        .captures(&body).ok_or(error::message("No model found"))?
        .get(1).unwrap()
        .as_str()
        .trim();

    let caps = V2_REGEX.captures_iter(&body);
    let pdfs :Vec<download::PdfLink>= caps.map(|c| download::PdfLink{
        brand    : "Contura".to_string(),
        model    : model.to_string(),
        filename : c.get(3).unwrap().as_str().to_string(),
        url      : c.get(1).unwrap().as_str().to_string(),
    }).collect();

    for pdf in pdfs {
        let result = download::pdf(pdf);

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }
    
    download::save_url("Contura".to_string(), model.to_string(), url.to_string()).unwrap();
    Ok(())
}

/*
var getHtmlUrl = "/api/productdocuments?id=" + productDocumentContext.systemId + "&l=" + productDocumentContext.language + "&m=" + productDocumentContext.market + "&b=" + productDocumentContext.brand;		
var request = $.ajax({ method: "GET", url: getHtmlUrl,dataType:"json" });
*/
fn contura_get_pdf_links(client : &Client, system_id :&str, language: &str, market: &str, brand :&str, model :&str) -> Result<Vec<download::PdfLink>>{
    let api_link = format!("https://www.contura.eu/api/productdocuments?id={}&l={}&m={}&b={}", system_id, language, market, brand);
    println!("link: {:?}", api_link);
    let body = client.get(api_link).send()?.text()?;
    
    let val:serde_json::Value = serde_json::from_str(&body)?;
    let docs = val.get("documents").unwrap().as_array().unwrap();
    
    let mut result:Vec<download::PdfLink> = vec![];
    for doc in docs {
        let d : ConturaApiDocumentJSON=serde_json::from_value(doc.to_owned())?;
        println!("{}", d.documentUrl);
        if d.documentUrl.ends_with(".pdf") {
            result.push(download::PdfLink{ 
                brand    : "Contura".to_string(),
                model    : model.to_string(),
                filename : d.description, 
                url      : fix_url(&d.documentUrl)
            });
        }
    }
    Ok(result)
}

fn fix_url(url: &str) -> String {
    if url.starts_with("https://www.contura.eu") {
        url.to_string()
    } else {
        format!("https://www.contura.eu{}", url)
    }
}

impl From<serde_json::Value> for ConturaKaminJSON {
    fn from(json: serde_json::Value) -> Self {
        serde_json::from_value(json).unwrap()
    }
}

impl From<serde_json::Value> for ConturaApiDocumentJSON {
    fn from(json: serde_json::Value) -> Self {
        serde_json::from_value(json).unwrap()
    }
}