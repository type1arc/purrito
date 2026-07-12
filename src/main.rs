use bytes::Bytes;
use clap::Parser;
use reqwest;
use serde::Deserialize;
use std::process::Command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Deserialize)]
struct WaifuItem {
    url: String,
}

#[derive(Debug, Deserialize)]
struct WaifuResponse {
    items: Vec<WaifuItem>,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    count: u32,
    #[arg(short, long)]
    res: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    for i in 1..=args.count {
        let response = reqwest::get("https://api.waifu.im/images?IncludedTags=waifu")
            .await?
            .json::<WaifuResponse>()
            .await?;
        let url = response.items.into_iter().next()
            .ok_or("no images returned")?.url;
        println!("caught waifu: {}", url);

        let filename = format!("waifu#{}.jpg", i);

        let img_bytes: Bytes = reqwest::get(&url).await?.bytes().await?;
        let mut file = File::create(filename.clone()).await?;
        file.write_all(&img_bytes).await?;

        Command::new("chafa")
            .args(&["--align=mid,mid", &filename])
            .arg(format!("--size={}", &args.res))
            .status()?;

        println!("bagged waifu: {}", &filename);
        println!("");
    }

    Ok(())
}
