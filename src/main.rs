use bytes::Bytes;
use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Backend {
    Feh,
    Catimg,
    Chafa,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Backend::Feh => "feh",
            Backend::Catimg => "catimg",
            Backend::Chafa => "chafa",
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
    #[arg(short, long, value_enum, default_value_t = Backend::Feh)]
    backend: Backend,
    #[arg(long)]
    no_write: bool,
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

fn ensure_floating() {
    let run = |cmd: &str, args: &[&str]| {
        let _ = Command::new(cmd).args(args).status();
    };

    if std::env::var_os("SWAYSOCK").is_some() {
        run("swaymsg", &["for_window", "[class=\"feh\"]", "floating", "enable"]);
    } else if std::env::var_os("I3SOCK").is_some() {
        run("i3-msg", &["for_window", "[class=\"feh\"]", "floating", "enable"]);
    } else if std::env::var_os("BSPWM_SOCKET").is_some() {
        run("bspc", &["rule", "-a", "feh", "-o", "state=floating"]);
    } else if std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|d| d.to_lowercase().contains("river")) {
        run("riverctl", &["float-filter-add", "app-id", "feh"]);
    }
}

fn feh_windows() -> Vec<(String, bool)> {
    let Ok(out) = Command::new("hyprctl").arg("-j").arg("clients").output() else {
        return vec![];
    };
    let Ok(text) = String::from_utf8(out.stdout) else { return vec![] };
    let Ok(clients) = serde_json::from_str::<Value>(&text) else { return vec![] };
    clients
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|c| c.get("class").and_then(|v| v.as_str()) == Some("feh"))
                .map(|c| {
                    (
                        c.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        c.get("floating").and_then(|v| v.as_bool()).unwrap_or(false),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn hyprland_float(addr: &str) {
    let lua = Command::new("hyprctl")
        .arg("dispatch")
        .arg(format!(
            "hl.dsp.window.float({{window = 'address:{addr}', action = 'toggle'}})"
        ))
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if lua && feh_windows().iter().any(|(a, f)| a == addr && *f) {
        return;
    }
    let _ = Command::new("hyprctl")
        .arg("dispatch")
        .arg(format!("togglefloating address:{addr}"))
        .status();
}

fn float_feh_hyprland() {
    thread::spawn(|| {
        for _ in 0..12 {
            thread::sleep(Duration::from_millis(250));
            for (addr, floating) in feh_windows() {
                if !floating {
                    hyprland_float(&addr);
                }
            }
            if feh_windows().iter().all(|(_, f)| *f) {
                return;
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
    tokio::fs::create_dir_all(&dir).await?;

    let client = Client::builder().user_agent("purrito").build()?;
    if args.backend == Backend::Feh {
        ensure_floating();
    }
    println!("source: {} | backend: {}", args.source.name(), args.backend.name());

    for i in 1..=args.count {
        let url = fetch_url(&client, args.source).await.map_err(|e| {
            format!("source {} failed: {e} (try --source nekos-best or --source waifu-pics)", args.source.name())
        })?;
        println!("caught waifu: {url}");

        let img_bytes: Bytes = client.get(&url).send().await?.bytes().await?;

        let path = if args.no_write {
            // viewers need a file; park bytes in the OS temp dir instead of the download dir
            std::env::temp_dir().join(format!("purrito-waifu#{}-{}.{}", i, std::process::id(), ext_of(&url)))
        } else {
            dir.join(format!("waifu_{i}.{}", ext_of(&url)))
        };
        let mut file = File::create(&path).await?;
        file.write_all(&img_bytes).await?;

        let width = args.res.split('x').next().unwrap_or(&args.res).to_string();
        let result = match args.backend {
            Backend::Feh => Command::new("feh")
                .arg(&path)
                .spawn()
                .map(|_| ()),
            Backend::Catimg => Command::new("catimg")
                .arg(format!("-w{width}"))
                .arg(&path)
                .status()
                .map(|_| ()),
            Backend::Chafa => Command::new("chafa")
                .arg(format!("--size={}", args.res))
                .arg("--align=mid,mid")
                .arg(&path)
                .status()
                .map(|_| ()),
        };

        if let Err(_) = result {
            println!("({} not found - skipping render)", args.backend.name());
        }

        if args.backend == Backend::Feh && std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            float_feh_hyprland();
        }

        println!("bagged waifu: {}", if args.no_write { "(no-write, parked in tmp)".to_string() } else { path.display().to_string() });
        println!();
    }

    Ok(())
}
