use regex::{Regex, RegexBuilder};
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};

use crate::error::{Result, self};
use crate::download;

lazy_static!{
    static ref REACT_ISTATE_REGEX : Regex = RegexBuilder::new("AppRegistry.registerInitialState(.*?),(?P<json>.*?)\\);</script")
        .build().unwrap();
}

pub fn run() {
    nordpeis_download().unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct NordpeisDocumentJSON {
    name :String,
    url :String,
}

fn nordpeis_download() -> Result<()> {
    let pages = vec![
        "https://www.nordpeis.com/se/alla-produkter/murspis-och-kamin",
    ];

    let client = Client::new();
    for page in pages {
        let body = client.get(page).send()?.text()?;

        let states = find_react_inital_states(&body);

        let products = states
            .iter()
            .filter(|s| s.starts_with("{\"data\""))
            .nth(0)
            .expect("did not find data");

        println!("extract {}", products);

        let json : serde_json::Value = serde_json::from_str(products)?;

        let prodjson = json
            .get("data").unwrap()
            .get("Items").unwrap()
            .as_array().unwrap();

        for p in prodjson {
            let url = p
                .get("url").unwrap()
                .as_str().unwrap();

            println!("url {}", &url);
            match nordpeis_single_page(&client, &url) {
                Ok(_) => (),
                Err(e)=> println!("Failed to download url: {} {:?}", &url, e),
            };
        }
    }

    Ok(())
}

fn nordpeis_single_page(client :&Client, url :&str) -> Result<()> {
    let body = client.get(url).send()?.text()?;
    let states = find_react_inital_states(&body);
    
    //find the product state
    let istate = states
        .iter()
        .filter(|s| s.starts_with("{\"data\":{\"productName"))
        .nth(0)
        .ok_or(error::message("did not find react inital state"))?;

    let json:serde_json::Value = serde_json::from_str(istate)?;
    let product = json
        .get("data")
        .ok_or(error::message("no product data found"))?;

    let name = product
        .get("productName").ok_or(error::message("no product name found"))?
        .as_str().ok_or(error::message("product name is not a str"))?;
    
    let docs = product.get("documents").ok_or(error::message("did not find documents"))?;

    let man_list = docs
        .get("otherManuals").ok_or(error::message("otherManuals not found"))?
        .get("list").ok_or(error::message("list not found"))?
        .as_array().ok_or(error::message("list is not an array"))?;

    for item in man_list {
        let doc:NordpeisDocumentJSON = serde_json::from_value(item.to_owned()).unwrap();

        let result = download::pdf(download::PdfLink { 
            brand    : "Nordpeis".to_string(), 
            model    : name.trim().to_string(), 
            filename : doc.name, 
            url      : doc.url, 
        });

        if let Err(e) = result {
            println!("Failed to download pdf: {:?}", e);
        }
    }

    download::save_url("Nordpeis".to_string(), name.trim().to_string(), url.to_string()).unwrap();
    Ok(())
}

fn find_react_inital_states(body :&str) -> Vec<&str> {
    REACT_ISTATE_REGEX.captures_iter(body)
        .map(|c| c.get(2).unwrap().as_str())
        .collect()
}