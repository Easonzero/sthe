//! A library to provide an easy way to extract data from HTML.

use anyhow::{anyhow, Result};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The configurable option for extracting
#[derive(Deserialize)]
pub struct ExtractOpt {
    #[serde(default)]
    pub target: Option<String>,
    pub selector: String,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default, flatten)]
    pub items: HashMap<String, ExtractOpt>,
}

pub struct ExtractOptCompled {
    pub target: Option<String>,
    pub selector: Selector,
    pub regex: Option<Regex>,
    pub items: HashMap<String, ExtractOptCompled>,
}

impl ExtractOpt {
    pub fn compile(self) -> Result<ExtractOptCompled> {
        Ok(ExtractOptCompled {
            target: self.target,
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
    #[serde(skip_serializing_if = "HashMap::is_empty", flatten)]
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
            .target
            .as_ref()
            .and_then(|target| match target.as_str() {
                "text" => Some(elem.text().collect::<Vec<_>>().join("")),
                attr => elem.value().attr(attr).map(|x| x.to_owned()),
            })
            .and_then(|text| {
                Some(if let Some(regex) = opt.regex.as_ref() {
                    let captures: Vec<_> = regex
                        .captures(&text.trim())?
                        .iter()
                        .skip(1)
                        .flat_map(|x| x.map(|x| x.as_str().to_owned()))
                        .collect();
                    if captures.len() == 1 {
                        ExtractText::Text(captures.into_iter().next().unwrap())
                    } else {
                        ExtractText::TextList(captures)
                    }
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

    macro_rules! test_case {
        (html:$html: literal, opt:$opt: literal, expect:$expect: literal) => {
            let opt: ExtractOpt = toml::from_str($opt).unwrap();
            let extract = extract_fragment($html, &opt.compile().unwrap());
            let extract_value = toml::Value::try_from(extract).unwrap();
            let expect_value = toml::from_str($expect).unwrap();
            assert_eq!(extract_value, expect_value);
        };
    }

    #[test]
    fn test_basis() {
        test_case! {
            html:"<a href=\"www.xxx.com\">",
            opt:r#"
                target = "href"
                selector = "a"
            "#,
            expect:"text = \"www.xxx.com\""
        };
    }

    #[test]
    fn test_recursive() {
        test_case! {
            html: r#"
<div class="parent"> Hello, <h2>world!</h2> </div>
<h2>Hello, world!</h2>               
            "#,
            opt: r#"
                selector = ".parent"

                [title]
                target = "text"
                selector = "h2"
            "#,
            expect: r#"
                [title]        
                text = "world!"
            "#
        };
    }

    #[test]
    fn test_list() {
        test_case! {
            html: r#"
<div class="parent"> Hello, <h2>w</h2>o<h2>r</h2>l<h2>d</h2>! </div>
            "#,
            opt: r#"
                selector = ".parent"

                [title]
                target = "text"
                selector = "h2"
            "#,
            expect: r#"
                [[title]]
                text = "w"
                [[title]]
                text = "r"
                [[title]]
                text = "d"
            "#
        };
    }

    #[test]
    fn test_capture() {
        test_case! {
            html: r#"
<div class="parent"> Hello, <h2>world!</h2> </div>
            "#,
            opt: r#"
                target = "text"
                selector = ".parent"
                regex = "(.*?), (.*?)!"
            "#,
            expect: r#"
                text = ["Hello", "world"]
            "#
        };
    }
}
