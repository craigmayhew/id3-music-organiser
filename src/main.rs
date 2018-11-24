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

//todo only include this "use" for running tests
use std::io::Write;

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

				let artist = artist(tag.artist());
                println!("  Artist Tag: {}", artist);

                let album: String = album(tag.album(),tag.album_artist());
				println!("  Album Tag: {}", &album);

                let title = RE_GOOD_CHARS_ONLY.replace_all(tag.title().unwrap(), "").to_string();
                println!("  Title: {}", title);

                let mut destination_path_with_file_name: String = destination_path_with_file_name(path, &artist, &album, &title);
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

fn destination_path_with_file_name(path: &Path, artist: &str, album: &str, title: &str) -> std::string::String {
	let mut destination_filename: String = "".to_owned();
	destination_filename.push_str(&title);
	destination_filename.push_str(".mp3");

	let mut destination: String = "sorted".to_owned();
	destination.push_str("/");
	destination.push_str(&artist);
	destination.push_str("/");
	destination.push_str(&album);
	destination.push_str("/");

	if !Path::new(&destination).exists() {
        let result = std::fs::create_dir_all(&destination);
		match result {
		    Ok(o) => o,
			Err(_e) => panic!("create_dir_all failed with an error")
		};
    }

    destination.push_str(&destination_filename);
    let result = std::fs::copy(path, &destination);
	match result {
		Ok(o) => o,
		Err(_e) => panic!("std::fs::copy failed with an error")
	};

	//return new path and file name
	destination
}

#[cfg(test)]
mod destination_path_with_file_name {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

	fn setup () {
	    let sorted_path = Path::new("sorted/unittest");
	    if !sorted_path.exists() {
			let result = std::fs::create_dir_all("sorted/unittest");
			match result {
				Ok(o) => o,
				Err(_e) => panic!("create_dir_all failed with an error")
			};
		}

	    let mut file = std::fs::File::create("sorted/unittest/example.mp3").unwrap();
        file.write_all(b"Just unit testing!").unwrap();

		let unsorted_path = Path::new("unsorted/unittest");
	    if !unsorted_path.exists() {
			let result = std::fs::create_dir_all("unsorted/unittest");
			match result {
				Ok(o) => o,
				Err(_e) => panic!("create_dir_all failed with an error")
			};
		}
	}

	fn teardown () {
	    let result = std::fs::remove_dir_all("sorted/unittest");
		match result {
			Ok(o) => o,
			Err(_e) => panic!("remove_dir_all failed with an error")
		};
		let result = std::fs::remove_dir_all("unsorted/unittest");
		match result {
			Ok(o) => o,
			Err(_e) => panic!("remove_dir_all failed with an error")
		};
	}

    #[test]
	//Check we create an mp3 file, in the correct folder structure
    fn check_creates_file() {
	    setup();
		
		let path = Path::new("sorted/unittest/example.mp3");
        assert_eq!(destination_path_with_file_name(path, "artist", "album", "title"), "sorted/artist/album/title.mp3".to_owned());
		//todo add a test to check the sorted file is actually where it's meant to be!

		teardown();
	}
}

fn artist(tag_artist: Option<&str>) -> std::string::String {
	let mut artist: String;

	lazy_static! {
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ ]").unwrap();
	}

	if tag_artist.is_some() {
		artist = tag_artist.unwrap().to_string();
	} else {
		artist = "NA".to_string();
	}
	artist = RE_GOOD_CHARS_ONLY.replace_all(&artist, "").to_string();
	artist
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
        assert_eq!(artist(option_mock_id3_artist), "abc".to_string());
    }
	#[test]
	fn artist_id3_tags_not_readable() {
		let str_mock_artist = "".to_string();
		let option_mock_id3_artist = mock_id3_option(&str_mock_artist);
        assert_eq!(artist(option_mock_id3_artist), "NA".to_string());
    }

}

fn album(tag_album: Option<&str>, tag_album_artist: Option<&str>) -> std::string::String {
    let mut album: String;

	lazy_static! {
        static ref RE_REMOVE_FEATURING: Regex = Regex::new(r"featuring(.*)").unwrap();
		static ref RE_REMOVE_FEAT: Regex = Regex::new(r"feat\.(.*)").unwrap();
        static ref RE_GOOD_CHARS_ONLY: Regex = Regex::new(r"[^a-zA-Z0-9\-_ ]").unwrap();
	}

    if tag_album.is_some() {
        album = tag_album.unwrap().to_string();
    } else if tag_album_artist.is_some() {
        album = tag_album_artist.unwrap().to_string();
    } else {
        album = "NA".to_owned();
    }

    if album.contains("featuring") {
        album = RE_REMOVE_FEATURING.replace(&album, "").to_string();
    }
	if album.contains("feat.") {
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
        assert_eq!(album(option_mock_id3_album, option_mock_id3_album_artist), "abc".to_string());
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
}