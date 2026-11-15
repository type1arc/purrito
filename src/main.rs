use bytes::Bytes;
use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Source {
    WaifuIm,
    NekosBest,
    WaifuPics,
}

impl Source {
    fn name(self) -> &'static str {
        match self {
            Source::WaifuIm => "waifu.im",
            Source::NekosBest => "nekos.best",
            Source::WaifuPics => "waifu.pics",
        }
    }
}

#[derive(Debug, Deserialize)]
struct WaifuItem {
    url: String,
}

#[derive(Debug, Deserialize)]
struct WaifuResponse {
    items: Vec<WaifuItem>,
}

#[derive(Debug, Deserialize)]
struct NekosItem {
    url: String,
}

#[derive(Debug, Deserialize)]
struct NekosResponse {
    results: Vec<NekosItem>,
}

#[derive(Debug, Deserialize)]
struct WaifuPicsResponse {
    url: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    count: u32,
    #[arg(short, long, default_value = "100x100")]
    res: String,
    #[arg(short, long, value_enum, default_value_t = Source::WaifuIm)]
    source: Source,
    #[arg(short, long)]
    dir: Option<PathBuf>,
}

async fn fetch_url(client: &Client, source: Source) -> Result<String, Box<dyn std::error::Error>> {
    let url = match source {
        Source::WaifuIm => {
            let resp = client
                .get("https://api.waifu.im/images")
                .query(&[("IncludedTags", "waifu")])
                .send()
                .await?
                .json::<WaifuResponse>()
                .await?;
            resp.items.into_iter().next().ok_or("no images returned")?.url
        }
        Source::NekosBest => {
            let resp = client
                .get("https://nekos.best/api/v2/neko")
                .query(&[("amount", "1")])
                .send()
                .await?
                .json::<NekosResponse>()
                .await?;
            resp.results.into_iter().next().ok_or("no images returned")?.url
        }
        Source::WaifuPics => {
            client
                .get("https://api.waifu.pics/sfw/waifu")
                .send()
                .await?
                .json::<WaifuPicsResponse>()
                .await?
                .url
        }
    };
    Ok(url)
}

fn ext_of(url: &str) -> String {
    let last = url.rsplit('/').next().unwrap_or("");
    match last.rsplit_once('.') {
        Some((_, e))
            if !e.is_empty() && e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            e.to_lowercase()
        }
        _ => "jpg".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
    tokio::fs::create_dir_all(&dir).await?;

    let client = Client::builder().user_agent("purrito").build()?;
    println!("source: {}", args.source.name());

    for i in 1..=args.count {
        let url = fetch_url(&client, args.source).await.map_err(|e| {
            format!("source {} failed: {e} (try --source nekos-best or --source waifu-pics)", args.source.name())
        })?;
        println!("caught waifu: {url}");

        let path = dir.join(format!("waifu#{i}.{}", ext_of(&url)));
        let img_bytes: Bytes = client.get(&url).send().await?.bytes().await?;
        let mut file = File::create(&path).await?;
        file.write_all(&img_bytes).await?;

        let width = args.res.split('x').next().unwrap_or(&args.res).to_string();
        let status = Command::new("catimg")
            .arg(format!("-w{width}"))
            .arg(&path)
            .status();

        match status {
            Ok(_) => {}
            Err(_) => println!("(catimg not found - skipping render)"),
        }

        println!("bagged waifu: {}", path.display());
        println!();
    }

    Ok(())
}
