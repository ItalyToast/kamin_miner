use reqwest::blocking::Client;
use serde_xml;
use serde_xml::value::{Content, Element};

use crate::error::MinerError;

pub fn get_pages(url: &str) -> crate::Result<Vec<String>>{
    let client = Client::new();

    let response = client.get(url)
        .send()?;

    if !response.status().is_success() {
        println!("[Sitemap][ERROR]{} {}", url, response.status().as_u16());
        return Ok(vec![]);
    }

    let body = response.text()?;
    let urlset: Element = match serde_xml::from_str(&body) {
        Ok(xml) => xml,
        Err(_) => return Err(MinerError::Message(format!("Could not parse XML at {}:\n{}", url, body).to_string())),
    };
    
    let mut result: Vec<String> = vec![];
    if let Content::Members(url_list) = urlset.members {
        for (tag, elements) in url_list {
            match tag.as_str() {
                "url" => {
                    for u in elements {
                        if let Some(loc) = u.attributes.get("loc"){
                            result.push(loc.first().unwrap().to_string());
                        }
                    }
                }
                "sitemap" => {
                    for u in elements {
                        if let Some(loc) = u.attributes.get("loc"){
                            match get_pages(loc.first().unwrap().as_str()) {
                                Ok(mut pages) => result.append(&mut pages),
                                Err(ex) => return Err(ex),
                            }
                        }
                    }
                }
                _ => ()
            }
        }
    }
    Ok(result)
}