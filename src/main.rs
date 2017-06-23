/* Use clap for arg parsing */
extern crate clap;

use std::io::{Read, Result, Error, ErrorKind};
use std::fs::File;
use clap::{Arg, App};

fn load_urls_from_file(path: &str) -> Result<Vec<String>> {
    /* First, we will open the path given to us */
    let path = std::path::Path::new(path);

    let mut file = match File::open(&path) {
        Err(why) => return Err(why),
        Ok(file) => file
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => return Err(why),
        _ => ()
    };

    let mut url_list = Vec::new();
    for url in s.split("\n") {
        url_list.push(url.to_owned());
    }

    Ok(url_list)
}

fn get_filenames_in_dir(path: &str) -> Result<Vec<String>> {
    let path = std::path::Path::new(path);
    if !path.exists() {
        match std::fs::create_dir(path) {
            Err(why) => return Err(why),
            _ => ()
        };
    }

    let filenames = match std::fs::read_dir(path) {
        Err(why) => return Err(why),
        Ok(entry) => entry
    };

    let mut path_list = Vec::new();
    for filename in filenames {
        let str_filename = match filename {
            Err(why) => return Err(why),
            Ok(entry) => match entry.file_name().into_string() {
                Err(_) => return Err(Error::new(ErrorKind::Other, "Uninterpretable filename")),
                Ok(converted_str) => converted_str
            }
        };

        path_list.push(str_filename);
    }

    Ok(path_list)
}


fn main() {
    let matches = App::new("wallers")
        .version("0.1")
        .about("A wallpaper setter and getter")
        .author("brandonio21")
        .arg(Arg::with_name("urlfile")
            .short("u")
            .long("urlfile")
            .value_name("URLFILE")
            .help("Path to file with list of newline delineated URLs")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("imagedir")
            .short("d")
            .long("imagedir")
            .value_name("IMAGEDIR")
            .help("Path to directory to store downloaded images")
            .takes_value(true)
            .required(true))
        .get_matches();

    let urlfile = matches.value_of("urlfile").unwrap();
    let imagedir = matches.value_of("imagedir").unwrap();

    /* Step 1: Load all URLs from the URLs file */
    let urls = match load_urls_from_file(&urlfile) {
        Err(why) => panic!("Error loading URLS from file {} : {}", urlfile, why),
        Ok(urls) => urls
    };

    /* Step 2: Load files from imagedir as cache */
    let cache_hashes = match get_filenames_in_dir(&imagedir) {
        Err(why) => panic!("Error reading files in image dir {} : {}", imagedir, why),
        Ok(cache) => cache
    };

    /* Step 3: Select a local image to display */

    /* Step 4: Decide whether we'll be trying to load a local image or URL */

    /* Step 5: 
    If local image, set wallpaper to selected image 
    If remote image, attempt to download and set. If fail, fallback to local image.
        If there is no local image, fall through
    */
}