# Purrito

A CLI tool written in Rust that fetches random waifu images from waifu.im / nekos.best / waifu.pics, saves them to disk, and renders them in the terminal via [catimg](https://github.com/posva/catimg).

## Dependencies

- [Rust](https://www.rust-lang.org/) toolchain (1.85+)
- [catimg](https://github.com/posva/catimg) – terminal image viewer

```fish
sudo pacman -S catimg rust cargo rustc
```

## Installation

```fish
git clone https://github.com/type1arc/purrito && cd purrito
cargo build --release
```

## Usage

```fish
cargo run --release -- --count <N> [--source <SOURCE>] [--res <WxH>] [--dir <DIR>]
```

- `--count` - number of waifus to fetch (default: 1)
- `--source` - image source: `waifu-im` (default), `nekos-best`, or `waifu-pics`
- `--res` - terminal render width passed to catimg, `WxH` or `W` (default: `100x100` → width 100)
- `--dir` - download directory (default: current directory, created if missing)

Each image is saved as `waifu#<N>.<ext>` in the download dir and displayed inline in the terminal via catimg.

## How it works

1. Parses args via [clap](https://github.com/clap-rs/clap).
2. For each iteration, requests a random image URL from the selected source API.
3. Downloads the image bytes and writes them to `<dir>/waifu#<i>.<ext>`.
4. Spawns `catimg -w<width> <path>` to render it in the terminal.

## License

MIT
