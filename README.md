# Purrito

A CLI tool written in Rust that fetches random waifu images from [waifu.im](https://waifu.im/), saves them to disk, and renders them in the terminal via [chafa](https://hpjansson.org/chafa/).

## Dependencies

- [Rust](https://www.rust-lang.org/) toolchain (1.85+)
- [chafa](https://hpjansson.org/chafa/) – terminal image viewer

```fish
sudo pacman -S chafa rust cargo rustc
```

## Installation

```fish
git clone https://github.com/type1arc/purrito && cd purrito
cargo build --release
```

## Usage

```fish
cargo run --release -- --count <N>
```

Replace `<N>` with the number of waifus you want. Each image is downloaded as `waifu#<N>` in the current directory and displayed inline in the terminal at 100×100 resolution.

## How it works

1. Parses `--count` via [clap](https://github.com/clap-rs/clap).
2. For each iteration, requests `https://api.waifu.im/images?IncludedTags=waifu` and extracts the first image URL from the JSON response.
3. Downloads the image bytes and writes them to `waifu#<i>`.
4. Spawns `chafa --size=100x100 <filename>` to render it in the terminal.

## License

MIT
