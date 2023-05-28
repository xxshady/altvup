use std::fs;

use colored::*;
use reqwest::{blocking::Client, header::USER_AGENT};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    name: String,
    prerelease: bool,
    assets: Vec<GithubAsset>,
}

fn main() {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();

    print_altvup();

    let mut args = std::env::args();
    args.next().unwrap();
    args.next().unwrap();

    let branch = args
        .next()
        .expect("Expected branch, example usage: cargo altvup release");

    let client = Client::new();

    println!(
        "Selected {} branch: {}",
        "alt:V".bright_green(),
        branch.bright_purple()
    );

    match branch.as_str() {
        "dev" => {
            let releases = get_releases(&client);
            let prerelease = releases
                .into_iter()
                .find(|v| v.prerelease)
                .expect("Cannot find prerelease in releases of altv-rust repo");

            download_module_from_release(&client, prerelease);
        }
        "release" => {
            let releases = get_releases(&client);
            let release = releases
                .into_iter()
                .find(|v| !v.prerelease)
                .expect("Cannot find release in releases of altv-rust repo");
            download_module_from_release(&client, release);
        }
        branch => panic!("Unsupported branch: {branch:?} use release or dev"),
    }

    print_altvup();
}

fn get_releases(client: &Client) -> Vec<GithubRelease> {
    let url = "https://api.github.com/repos/xxshady/altv-rust/releases";

    println!(
        "Loading releases data from {} repo",
        "altv-rust".bright_green()
    );

    let res = client
        .get(url)
        .header(USER_AGENT, "rust-altv-updater")
        .send()
        .unwrap_or_else(|e| {
            panic!("Failed to get github releases data from: {url}, error: {e}");
        });
    res.json().unwrap_or_else(|e| {
        panic!("Failed to convert github releases data to json from: {url}, error: {e}");
    })
}

fn download_module_from_release(client: &Client, release: GithubRelease) {
    println!(
        "Downloading Rust module from release: {}",
        release.name.bright_blue()
    );

    let asset = release
        .assets
        .iter()
        .find(|v| v.name.ends_with(if cfg!(windows) { ".dll" } else { ".so" }))
        .unwrap_or_else(|| {
            panic!("Cannot find asset in release: {release:?} of altv-rust repo");
        });

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to download asset: {}, error: {e}",
                asset.browser_download_url
            );
        })
        .bytes()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to convert asset: {} to bytes, error: {e}",
                asset.browser_download_url
            );
        });

    fs::create_dir_all("modules").expect("Failed to create modules directory");

    let path = format!("modules/{}", asset.name);

    println!("Saving to: {}", path.bright_cyan());

    fs::write(&path, bytes).unwrap_or_else(|e| {
        panic!("Failed to write: {path}, error: {e}");
    });

    println!("{}", "Successfully saved".bright_green());
}

fn print_altvup() {
    println!(
        "====== {}{} ======",
        "altv".bright_green(),
        "up".bright_cyan()
    );
}
