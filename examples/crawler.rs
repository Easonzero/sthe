use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;

#[derive(Deserialize)]
struct Config {
    url: String,
    #[serde(flatten)]
    items: HashMap<String, sthe::ExtractOpt>,
}

#[derive(Serialize)]
struct Output {
    #[serde(flatten)]
    items: HashMap<String, sthe::Extract>,
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[arg(long, short)]
    config: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut config: Config = toml::from_str(&std::fs::read_to_string(&args.config).unwrap())
        .expect("invalid config format");

    let html = reqwest::get(&config.url)
        .await
        .expect("http request fault")
        .text()
        .await
        .expect("http request fault");

    let opts: HashMap<_, _> = std::mem::take(&mut config.items)
        .into_iter()
        .map(|(k, opt)| (k, opt.compile().unwrap()))
        .collect();

    let mut outs = HashMap::new();

    for (k, opt) in opts.into_iter() {
        outs.insert(k, sthe::extract_document(&html, &opt));
    }

    let out = Output { items: outs };
    println!("{}", toml::to_string_pretty(&out).unwrap());
}
