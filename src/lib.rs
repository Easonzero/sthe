//! A library to provide an easy way to extract data from HTML.

use anyhow::{anyhow, Result};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(tag = "type", content = "index")]
pub enum Label {
    Text,
    Attr(String),
}

/// The configurable option for extracting
#[derive(Deserialize)]
pub struct ExtractOpt {
    #[serde(default)]
    pub label: Option<Label>,
    pub selector: String,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default, flatten)]
    pub items: HashMap<String, ExtractOpt>,
}

pub struct ExtractOptCompled {
    pub label: Option<Label>,
    pub selector: Selector,
    pub regex: Option<Regex>,
    pub items: HashMap<String, ExtractOptCompled>,
}

impl ExtractOpt {
    pub fn compile(self) -> Result<ExtractOptCompled> {
        Ok(ExtractOptCompled {
            label: self.label,
            selector: Selector::parse(&self.selector).map_err(|e| anyhow!("{:?}", e))?,
            regex: self.regex.map(|x| Regex::new(&x)).transpose()?,
            items: self
                .items
                .into_iter()
                .map(|(k, v)| Ok((k, v.compile()?)))
                .collect::<Result<_>>()?,
        })
    }
}

/// The text extracted
#[derive(Serialize)]
#[serde(untagged)]
pub enum ExtractText {
    Text(String),
    TextList(Vec<String>),
}

/// The item in result extracted
#[derive(Serialize)]
pub struct ExtractItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<ExtractText>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    items: HashMap<String, Extract>,
}

/// The result extracted
#[derive(Serialize)]
#[serde(untagged)]
pub enum Extract {
    Item(ExtractItem),
    ItemList(Vec<ExtractItem>),
}

fn extract_elem(elem: ElementRef, opt: &ExtractOptCompled) -> Extract {
    let select = elem.select(&opt.selector);
    let mut extract_items = vec![];
    for elem in select {
        let text = opt
            .label
            .as_ref()
            .and_then(|label| match label {
                Label::Text => Some(elem.text().collect::<Vec<_>>().join("")),
                Label::Attr(k) => elem.value().attr(k).map(|x| x.to_owned()),
            })
            .and_then(|text| {
                Some(if let Some(regex) = opt.regex.as_ref() {
                    ExtractText::TextList(
                        regex
                            .captures(&text.trim())?
                            .iter()
                            .skip(1)
                            .flat_map(|x| x.map(|x| x.as_str().to_owned()))
                            .collect(),
                    )
                } else {
                    ExtractText::Text(text.trim().to_owned())
                })
            });
        let items: HashMap<_, _> = opt
            .items
            .iter()
            .map(|(k, v)| (k.clone(), extract_elem(elem, v)))
            .collect();
        extract_items.push(ExtractItem { text, items });
    }

    if extract_items.len() == 1 {
        Extract::Item(extract_items.into_iter().next().unwrap())
    } else {
        Extract::ItemList(extract_items)
    }
}

fn extract_html(html: Html, opt: &ExtractOptCompled) -> Extract {
    let root_elem = html.root_element();
    extract_elem(root_elem, opt)
}

/// Extract from a string of document.
pub fn extract_document(document: &str, opt: &ExtractOptCompled) -> Extract {
    let document = Html::parse_document(document);
    extract_html(document, opt)
}

/// Extract from a string of fragment.
pub fn extract_fragment(fragment: &str, opt: &ExtractOptCompled) -> Extract {
    let fragment = Html::parse_fragment(fragment);
    extract_html(fragment, opt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basis() {
        let opt = ExtractOpt {
            label: Some(Label::Attr("href".to_owned())),
            selector: "a".to_owned(),
            regex: None,
            items: Default::default(),
        };
        let extract = extract_fragment("<a href=\"www.xxx.com\"/>", &opt.compile().unwrap());
        match extract {
            Extract::Item(ExtractItem {
                text: Some(ExtractText::Text(text)),
                ..
            }) => {
                assert_eq!(&text, "www.xxx.com")
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_recursive() {
        let d_opt = ExtractOpt {
            label: Some(Label::Text),
            selector: "h2".to_owned(),
            regex: None,
            items: Default::default(),
        };
        let opt = ExtractOpt {
            label: None,
            selector: ".parent".to_owned(),
            regex: None,
            items: [("title".to_owned(), d_opt)].into_iter().collect(),
        };

        let extract = extract_fragment(
            r#"
<!DOCTYPE html>
<div class="parent"> Hello, <h2>world!</h2> </div>
<h2>Hello, world!</h2>
            "#,
            &opt.compile().unwrap(),
        );
        match extract {
            Extract::Item(ExtractItem { items, .. }) => {
                assert!(items.contains_key("title"));

                match &items["title"] {
                    Extract::Item(ExtractItem {
                        text: Some(ExtractText::Text(text)),
                        ..
                    }) => {
                        assert_eq!(text, "world!");
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_list() {
        let opt = ExtractOpt {
            label: Some(Label::Text),
            selector: ".parent".to_owned(),
            regex: None,
            items: Default::default(),
        };
        let extract = extract_fragment(
            r#"
<!DOCTYPE html>
<div class="parent"> Hello, <h2>world!</h2> </div>
<div class="parent"> Hello, <h2>world!</h2> </div>
<div class="parent"> Hello, <h2>world!</h2> </div>
            "#,
            &opt.compile().unwrap(),
        );

        match extract {
            Extract::ItemList(list) => {
                assert_eq!(list.len(), 3);
                let extract = &list[0];
                match &extract.text {
                    Some(ExtractText::Text(text)) => {
                        assert_eq!(text, "Hello, world!");
                    }
                    _ => assert!(false),
                }
            }
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_capture() {
        let opt = ExtractOpt {
            label: Some(Label::Text),
            selector: ".parent".to_owned(),
            regex: Some("(.*?), (.*?)!".to_owned()),
            items: Default::default(),
        };
        let extract = extract_fragment(
            r#"
<!DOCTYPE html>
<div class="parent"> Hello, <h2>world!</h2> </div>
            "#,
            &opt.compile().unwrap(),
        );

        match extract {
            Extract::Item(ExtractItem {
                text: Some(ExtractText::TextList(texts)),
                ..
            }) => {
                assert_eq!(texts.len(), 2);
                assert_eq!(&texts[0], "Hello");
                assert_eq!(&texts[1], "world");
            }
            _ => {
                assert!(false)
            }
        }
    }
}
