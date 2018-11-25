// required to allow us to create static regex and save cpu
#[macro_use]
extern crate lazy_static;
//required for colours in our command line output
extern crate colored;
//required to read id3 tags from files
extern crate id3;
//required for any regex we run
extern crate regex;
//required to walk through a directory structure
extern crate walkdir;
//required by decopt
#[macro_use]
extern crate serde_derive;
//required for accepting neat command line arguements
extern crate docopt;

use colored::*;
use docopt::Docopt;
use regex::Regex;
use regex::RegexBuilder;
use std::fs;
use std::fs::File;
#[cfg(test)]
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

const USAGE: &'static str = "
ID3 Music Organiser.

Usage:
  id3org
  id3org <unorganised> <organised>
  id3org -h
  id3org --help
  id3org --skipalbums
  id3org --version

Options:
  -h --help         Show this screen.
  --version         Show version.
  --skipalbums     Does not create album sub folders.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_skipalbums: bool,
    arg_organised: Vec<String>,
	arg_unorganised: Vec<String>,
}

fn main() -> std::io::Result<()> {
	let destination = "sorted".to_string();
    let mut file_mv_counter:i64 = 0;
    let mut file_skipped_counter:i64 = 0;

	/*
	  process command line arguements and switches
	*/
	let args: docopt::ArgvMap = parse_config();

	/*read through files, and copy them into new folder structure*/
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

				let artist = artist(tag.artist());
                println!("  Artist Tag: {}", artist);

				let album_name: String;
				if args.get_bool("--skipalbums") {
				    album_name = "".to_string();
				} else {
                    album_name = album(tag.album(),tag.album_artist());
				    println!("  Album Tag: {}", &album_name);
				}

				let title = title(tag.title());
                println!("  Title: {}", title);

                let mut destination_path_with_file_name: String = destination_path_with_file_name(path, &destination, &artist, &album_name, &title);
                println!("{}{}\n", "  COPYING FILE to ".cyan(), destination_path_with_file_name.cyan());

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

//parse command line arguements/switches
fn parse_config() -> (docopt::ArgvMap) {
	Docopt::new(USAGE)
		.and_then(|dopt| dopt.parse())
		.unwrap_or_else(|e| e.exit())
}

fn destination_path_with_file_name(path: &Path, destination_folder: &str, artist: &str, album: &str, title: &str) -> std::string::String {
	let mut destination_filename: String = "".to_owned();
	destination_filename.push_str(&title);
	destination_filename.push_str(".mp3");

	let mut destination: String = destination_folder.to_owned();
	destination.push_str("/");
	destination.push_str(&artist);
	destination.push_str("/");
	destination.push_str(&album);
	destination.push_str("/");

	if !Path::new(&destination).exists() {
		match std::fs::create_dir_all(&destination) {
		    Ok(o) => o,
			Err(_e) => panic!("create_dir_all failed with an error")
		};
    }

	//concat filename to the filepath
    destination.push_str(&destination_filename);

	//check if the file already exists
	if Path::new(&destination).is_file() {
		let sorted_file_metadata = match fs::metadata(&destination) {
			Ok(o) => o,
			Err(_e) => panic!("fs::metadata failed with an error")
		};
		let unsorted_file_metadata = match fs::metadata(&path) {
			Ok(o) => o,
			Err(_e) => panic!("fs::metadata failed with an error")
		};
	    //the file exists and we only want to overwrite it if the latest file is larger
		if unsorted_file_metadata.len() > sorted_file_metadata.len() {
			match std::fs::copy(path, &destination) {
				Ok(o) => o,
				Err(_e) => panic!("std::fs::copy failed with an error")
			};
		}
	} else {
		match std::fs::copy(path, &destination) {
			Ok(o) => o,
			Err(_e) => panic!("std::fs::copy failed with an error")
		};
	}

	//return new path and file name
	destination
}

#[cfg(test)]
mod destination_path_with_file_name {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

	fn setup () {
	    let sorted_path = Path::new("unittest-sorted");
	    if !sorted_path.exists() {
			match std::fs::create_dir_all("unittest-sorted") {
				Ok(o) => o,
				Err(_e) => panic!("create_dir_all failed with an error")
			};
		}

	    let mut file = std::fs::File::create("unittest-sorted/example.mp3").unwrap();
        file.write_all(b"Just unit testing!").unwrap();

		let unsorted_path = Path::new("unittest-unsorted");
	    if !unsorted_path.exists() {
			match std::fs::create_dir_all("unittest-unsorted") {
				Ok(o) => o,
				Err(_e) => panic!("create_dir_all failed with an error")
			};
		}
	}

	fn teardown () {
		match std::fs::remove_dir_all("unittest-sorted") {
			Ok(o) => o,
			Err(_e) => panic!("remove_dir_all failed with an error")
		};
		match std::fs::remove_dir_all("unittest-unsorted") {
			Ok(o) => o,
			Err(_e) => panic!("remove_dir_all failed with an error")
		};
	}

    #[test]
	//Check we create an mp3 file, in the correct folder structure
    fn check_creates_file() {
	    setup();
		
		let path = Path::new("unittest-sorted/example.mp3");
        assert_eq!(destination_path_with_file_name(path, "unittest-sorted", "artist", "album", "title"), "unittest-sorted/artist/album/title.mp3".to_owned());
		//todo add a test to check the sorted file is actually where it's meant to be!

		teardown();
	}
}

fn title(tag_title: Option<&str>) -> std::string::String {
    lazy_static! {
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ &]").unwrap();
	}
    let title = RE_GOOD_CHARS_ONLY.replace_all(tag_title.unwrap(), "").to_string();
	title
}

#[cfg(test)]
mod title {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

	fn mock_id3_option(input_string: &str) -> Option<&str> {
		if input_string == "" {
			None
		} else {
			Some(input_string)
		}
	}

	#[test]
	//Check we can read the title id3 tag
    fn title_basic() {
		let str_mock_title = "Test Title Name".to_string();
		let option_mock_id3_title = mock_id3_option(&str_mock_title);
        assert_eq!(artist(option_mock_id3_title), str_mock_title);
	}
	#[test]
	//check a bunch of bad characters are actually removed from the returned title name
	//e.g. "Title$1" becomes "Title1"
	fn title_remove_bad_chars() {
		let str_mock_title = "!,.<>/;:abc*&^".to_string();
		let option_mock_id3_title = mock_id3_option(&str_mock_title);
        assert_eq!(artist(option_mock_id3_title), "abc&".to_string());
    }
}

fn artist(tag_artist: Option<&str>) -> std::string::String {
	let mut artist: String;

	lazy_static! {
	    static ref RE_REMOVE_FEATURING: Regex = RegexBuilder::new(r"featuring(.*)")
		                                                      .case_insensitive(true)
															  .build()
															  .expect("Invalid Regex");
		static ref RE_REMOVE_FEAT: Regex = RegexBuilder::new(r"feat\.(.*)")
		                                                      .case_insensitive(true)
															  .build()
                                                              .expect("Invalid Regex");
	    static ref RE_REMOVE_FT: Regex = RegexBuilder::new(r" ft (.*)")
		                                                      .case_insensitive(true)
															  .build()
                                                              .expect("Invalid Regex");
		static ref RE_REMOVE_FT_DOT: Regex = RegexBuilder::new(r" ft\.(.*)")
		                                                      .case_insensitive(true)
															  .build()
                                                              .expect("Invalid Regex");
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ &]").unwrap();
	}

	if tag_artist.is_some() {
		artist = tag_artist.unwrap().to_string();
	} else {
		artist = "NA".to_string();
	}

	if artist.to_lowercase().contains("featuring") {
        artist = RE_REMOVE_FEATURING.replace(&artist, "").to_string();
    }
	if artist.to_lowercase().contains("feat.") {
        artist = RE_REMOVE_FEAT.replace(&artist, "").to_string();
    }
	if artist.to_lowercase().contains(" ft ") {
        artist = RE_REMOVE_FT.replace(&artist, "").to_string();
    }
	if artist.to_lowercase().contains(" ft.") {
        artist = RE_REMOVE_FT_DOT.replace(&artist, "").to_string();
    }

	artist = RE_GOOD_CHARS_ONLY.replace_all(&artist, "").to_string();
	artist.trim_right().to_string()
}

#[cfg(test)]
mod artist {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

	fn mock_id3_option(input_string: &str) -> Option<&str> {
		if input_string == "" {
			None
		} else {
			Some(input_string)
		}
	}

    #[test]
	//Check we can read the album id3 tag
    fn artist_basic() {
		let str_mock_artist = "Test Artist Name".to_string();
		let option_mock_id3_artist = mock_id3_option(&str_mock_artist);
        assert_eq!(artist(option_mock_id3_artist), str_mock_artist);
	}
	#[test]
	//check a bunch of bad characters are actually removed from the returned artist name
	//e.g. "Artist$1" becomes "Artist1"
	fn artist_remove_bad_chars() {
		let str_mock_artist = "!,.<>/;:abc*&^".to_string();
		let option_mock_id3_artist = mock_id3_option(&str_mock_artist);
        assert_eq!(artist(option_mock_id3_artist), "abc&".to_string());
    }
	#[test]
	fn artist_id3_tags_not_readable() {
		let str_mock_artist = "".to_string();
		let option_mock_id3_artist = mock_id3_option(&str_mock_artist);
        assert_eq!(artist(option_mock_id3_artist), "NA".to_string());
    }
	#[test]
	//check case that ft is truncated from artist names
	fn artist_remove_ft() {
		let str_mock_artist = "Someone ft someone else".to_string();
		let option_mock_id3_artist = mock_id3_option(&str_mock_artist);
        assert_eq!(artist(option_mock_id3_artist), "Someone".to_string());
    }
	#[test]
	//check case insensitivity of feat. featuring etc that are truncated from artist names
	fn artist_remove_feat_case_insensitive() {
		let str_mock_artist = "Someone FEat. someone else".to_string();
		let option_mock_id3_artist = mock_id3_option(&str_mock_artist);
        assert_eq!(artist(option_mock_id3_artist), "Someone".to_string());
    }
}

fn album(tag_album: Option<&str>, tag_album_artist: Option<&str>) -> std::string::String {
    let mut album: String;

	lazy_static! {
        static ref RE_REMOVE_FEATURING: Regex = RegexBuilder::new(r"featuring(.*)")
		                                                      .case_insensitive(true)
															  .build()
															  .expect("Invalid Regex");
		static ref RE_REMOVE_FEAT: Regex = RegexBuilder::new(r"feat\.(.*)")
		                                                      .case_insensitive(true)
															  .build()
                                                              .expect("Invalid Regex");
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ &]").unwrap();
	}

    if tag_album.is_some() {
        album = tag_album.unwrap().to_string();
    } else if tag_album_artist.is_some() {
        album = tag_album_artist.unwrap().to_string();
    } else {
        album = "NA".to_owned();
    }

    if album.to_lowercase().contains("featuring") {
        album = RE_REMOVE_FEATURING.replace(&album, "").to_string();
    }
	if album.to_lowercase().contains("feat.") {
        album = RE_REMOVE_FEAT.replace(&album, "").to_string();
    }
	album = RE_GOOD_CHARS_ONLY.replace_all(&album, "").to_string();

	album.trim_right().to_string()
}

#[cfg(test)]
mod album {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

	fn mock_id3_option(input_string: &str) -> Option<&str> {
		if input_string == "" {
			None
		} else {
			Some(input_string)
		}
	}

    #[test]
	//Check we can read the album id3 tag
    fn album_basic() {
		let str_mock_album = "Test Album Name".to_string();
		let str_mock_album_artist = "".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), str_mock_album);
	}
	#[test]
	//check we can read the album_artist which sometimes is actually the name of the album
	fn album_artist_basic() {
		let str_mock_album = "".to_string();
		let str_mock_album_artist = "Test Album Name".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), str_mock_album_artist);
	}
	#[test]
	//check a bunch of bad characters are actually removed from the returned album name
	//e.g. "Album$1" becomes "Album1"
	fn album_remove_bad_chars() {
		let str_mock_album = "!,.<>/;:abc*&^".to_string();
		let str_mock_album_artist = "".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), "abc&".to_string());
    }
	#[test]
	//check scenario where the album name can't be read
	fn album_id3_tags_not_readable() {
		let str_mock_album = "".to_string();
		let str_mock_album_artist = "".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), "NA".to_string());
    }
	#[test]
	//check feat. featuring etc are truncated from album names
	fn album_album_featuring() {
		let str_mock_album = "Something featuring someone".to_string();
		let str_mock_album_artist = "".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), "Something".to_string());
    }
	#[test]
	//check feat. featuring etc are truncated from album names
	fn album_album_artist_feat() {
		let str_mock_album = "".to_string();
		let str_mock_album_artist = "Something feat. someone".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), "Something".to_string());
    }
	#[test]
	//check canse insensitivity of feat. featuring etc that are truncated from album names
	fn album_album_artist_feat_case_insensitive() {
		let str_mock_album = "".to_string();
		let str_mock_album_artist = "Something FEat. someone".to_string();
		let option_mock_id3_album = mock_id3_option(&str_mock_album);
		let option_mock_id3_album_artist = mock_id3_option(&str_mock_album_artist);
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), "Something".to_string());
    }
}