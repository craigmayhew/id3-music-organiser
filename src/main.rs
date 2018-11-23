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
    let re_remove_featuring = Regex::new(r"featuring(.*)").unwrap();
    let re_good_chars_only = Regex::new(r"[^a-zA-Z0-9\-_ ]").unwrap();

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
                artist = re_good_chars_only.replace_all(&artist, "").to_string();
                println!("  Artist Tag: {}", artist);

                let mut album: String;
                if tag.album().is_some() {
                    album = tag.album().unwrap().to_string();
                } else if tag.album_artist().is_some() {
                    album = tag.album_artist().unwrap().to_string();
                } else {
                    album = "NA".to_owned();
                }
                album = re_good_chars_only.replace_all(&album, "").to_string();
                if album.contains("featuring") {
                    album = re_remove_featuring.replace(&album, "").to_string();
                }
                println!("  Album Tag: {}", &album);

                let title = re_good_chars_only.replace_all(tag.title().unwrap(), "").to_string();
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
