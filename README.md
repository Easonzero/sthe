# STHE
A library to provide an easy way to extract data from HTML.

## Example

```rust
// build extract option by toml
let opt: ExtractOpt = toml::from_str(r#"
  target = "href"
  selector = "a"
"#).unwrap();

// extract
let extract = extract_fragment("<a href=\"www.xxx.com\">", &opt.compile().unwrap());

// serialize result
let extract_value = toml::Value::try_from(extract).unwrap();
let expect_value = toml::from_str("text = \"www.xxx.com\"").unwrap();

assert_eq!(extract_value, expect_value);
```
