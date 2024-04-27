use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use clap::Parser;
use clap::ArgAction::SetTrue;
use colored::Colorize;
use regex::Regex;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config : Option<PathBuf>, // har file's path (option/ in default, uses "td.config"

    #[arg(short, long, action = SetTrue)]
    move_to : bool,

    #[arg(short, long, action = SetTrue)]
    uuid : bool,

}

fn main() -> Result<(), Box<dyn Error>> {
    let args : Args = Args::parse();
    let config : PathBuf = match args.config {
        None => if has_td_config() { Path::new("./td.config").to_path_buf() } else { return Err(Box::from("not found ./td.config"))},
        Some(p) => p
    };
    let move_to : bool = args.move_to;
    let to_uuid : bool = args.uuid;
    let mut har_file : String = "".to_string();

    let mut destination_dir : String = "".to_string();

    let destination_dir_regex : Regex = Regex::new("dir: .*").unwrap();
    let file_name_regex : Regex = Regex::new("file_name: ([a-zA-Z0-9_-]+)\\.har").unwrap();
    for line in BufReader::new(File::open(&config)?).lines() {
        let line : String = line?;
        if move_to && (&destination_dir_regex).is_match((&line).as_str()) {
            destination_dir = (*&line).replace("dir: ", "");
        };

        if !file_name_regex.is_match((&line).as_str()) { continue };
        har_file = line.replace("file_name: ", "");
        break;
    };
    if har_file.is_empty() {
        println!("{}", "Couldn't identify the target path.".red());
        return Err(Box::from("Couldn't identify the target path."));
    }

    if move_to && !Path::new(&destination_dir).exists() {
        return Err(Box::from("Failed to get the destination dir."));
    }
    let har_file : PathBuf = Path::new(&har_file).to_path_buf();
    if !&har_file.exists() { return Err(Box::from("target har file couldn't find.")); }

    let url_regex = match Regex::new(".*\"url\": \"(https://pbs.twimg.com/media/[a-zA-Z0-9_]+.*)\\?format=([a-z_]+).*\".*") {
        Ok(e) => e,
        Err(_e) => return Err(Box::from("url regex build error"))
    };

    let mut url_set : HashSet<String> = HashSet::new();
    for line in BufReader::new(File::open(&har_file)?).lines() {
        let line = line?;
        if !url_regex.is_match(&line) { continue };
        let capture = match url_regex.captures(&line) { Some(e) => e, None => continue };
        //if capture.len() != 2 { continue };
        let url = match  capture.get(1) { Some(e) => e.as_str(), None => continue };
        let extension = match capture.get(2) { Some(e) => e.as_str(), None => continue };
        url_set.insert(format!("{}.{}", url, extension));
    }

    let extensions : Vec<&str> = vec!["png", "jpg", "jpeg", "webp"];
    let url_regex : Regex = Regex::new("https://pbs.twimg.com/media/([a-zA-Z0-9_]+).([a-zA-Z_]+)").unwrap();
    for url in url_set {
        let img_bytes = reqwest::blocking::get(url.clone())?.bytes()?;
        let image = image::load_from_memory(&img_bytes)?;
        let capture = match url_regex.captures(&url.as_str()) { Some(e) => e, None => continue };
        let mut file_name = {
            if to_uuid { Uuid::new_v4().to_string() }
            else { match capture.get(1) {
                Some(e) => e.as_str().to_string(),
                None => continue,
            } }
        };
        file_name = format!("{}.{}", file_name, match capture.get(2) {
            Some(e) => {
                let extension : &str = e.as_str();
                if !extensions.contains(&extension) { continue; };
                extension
            },
            None => continue,
        });

        let mut save_place = if move_to { Path::new(&destination_dir).to_path_buf() } else { Path::new(".").to_path_buf() };
        save_place = save_place.join(Path::new(&file_name));
        image.save(&save_place).unwrap();
        println!("save as {}", &save_place.file_name().unwrap().to_str().unwrap().to_string());
    }

    return Ok(())

}

fn has_td_config() -> bool {
    return match Path::new("./td.config").try_exists() {
        Ok(_b) => true,
        Err(_e) => false
    }
}
