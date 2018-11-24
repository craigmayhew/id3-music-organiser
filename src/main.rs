#[macro_use] extern crate lazy_static;
extern crate colored;
extern crate id3;
extern crate regex;
extern crate walkdir;
use colored::*;
use regex::Regex;
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    lazy_static! {
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ ]").unwrap();
	}

    let mut file_mv_counter:i64 = 0;
    let mut file_skipped_counter:i64 = 0;

    for entry in WalkDir::new("./unsorted").into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue
        }

        let path = entry.path();
        let mut file = File::open(path)?;

        println!("\n{}: ", path.display());

        if let Ok(_res) = id3::Tag::is_candidate(file) {
            if let Ok(_res) = id3::Tag::read_from_path(path) {
                let tag = id3::Tag::read_from_path(path).unwrap();

                let mut artist: String;
                if tag.artist().is_some() {
                    artist = tag.artist().unwrap().to_string();
                } else {
                    artist = "NA".to_string();
                }
                artist = RE_GOOD_CHARS_ONLY.replace_all(&artist, "").to_string();
                println!("  Artist Tag: {}", artist);

                let album: String = album(tag.album(),tag.album_artist());

                let title = RE_GOOD_CHARS_ONLY.replace_all(tag.title().unwrap(), "").to_string();
                println!("  Title: {}", title);


                let mut destination_filename: String = "".to_owned();;
                destination_filename.push_str(&title);
                destination_filename.push_str(".mp3");

                let mut destination: String = "sorted".to_owned();
                destination.push_str("/");
                destination.push_str(&artist);
                destination.push_str("/");
                destination.push_str(&album);
                destination.push_str("/");

                println!("{}{}{}\n", "  COPYING FILE to ".cyan(), &destination.cyan(), destination_filename.cyan());

                if Path::new(&destination).exists() {

                } else {
                    std::fs::create_dir_all(&destination)?;
                }

                destination.push_str(&destination_filename);
                std::fs::copy(path, destination)?;

                file_mv_counter += 1;
            } else {
                println!("{}{:?}\n", "  NO ATTRIBUTES FOUND for ".red(), path.display());
                file_skipped_counter += 1;
            }

        } else {
            println!("{}{:?}\n", "  NO ATTRIBUTES FOUND for ".red(), path.display());
            file_skipped_counter += 1;
        }


    }
    println!("----------------------------------\n{}{}", "  FILES COPIED ".green().bold(), file_mv_counter.to_string().bold());
    println!("{}{}",   "  FILES SKIPPED ".green().bold(), file_skipped_counter.to_string().bold());
    Ok(())
}

fn album(tab_album: Option<&str>, tag_album_artist: Option<&str>) -> std::string::String {
    let mut album: String;

	lazy_static! {
        static ref RE_REMOVE_FEATURING: Regex = Regex::new(r"featuring(.*)").unwrap();
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ ]").unwrap();
	}

    if tab_album.is_some() {
        album = tab_album.unwrap().to_string();
    } else if tag_album_artist.is_some() {
        album = tag_album_artist.unwrap().to_string();
    } else {
        album = "NA".to_owned();
    }
    album = RE_GOOD_CHARS_ONLY.replace_all(&album, "").to_string();
    if album.contains("featuring") {
        album = RE_REMOVE_FEATURING.replace(&album, "").to_string();
    }
    println!("  Album Tag: {}", &album);
	album
}