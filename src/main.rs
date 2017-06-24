/* Use clap for arg parsing */
extern crate clap;
extern crate rand;
extern crate curl;
extern crate easy_hash;

use std::io::{Read, Error, ErrorKind, Write};
use std::fs::{create_dir, rename, read_dir};
use std::fs::File;
use clap::{Arg, App};
use std::path::Path;
use rand::{thread_rng, Rng};
use curl::easy::Easy;
use std::process::Command;
use easy_hash::{Sha256, Hasher, HashResult};

fn load_urls_from_file(path: &str) -> std::io::Result<Vec<String>> {
    /* First, we will open the path given to us */
    let path = std::path::Path::new(path);

    let mut file = match File::open(&path) {
        Err(why) => return Err(why),
        Ok(file) => file
    };

    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let mut url_list = Vec::new();
    for url in s.split_whitespace() {
        url_list.push(url.to_owned());
    }

    Ok(url_list)
}

fn get_filenames_in_dir(path: &Path) -> std::io::Result<Vec<String>> {
    if !path.exists() {
        create_dir(path)?;
    }

    let filenames = match read_dir(path) {
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

fn get_url_hash(url: &str) -> String {
    Sha256::hash(url.as_bytes()).hex()
}

fn download_remote_url(url: &str, destination: &Path) -> std::io::Result<()> {
    /* Append a "working" extension to the file */
    let temp_download_url_buf = destination.with_extension("download");
    let temp_download_url = temp_download_url_buf.as_path();

    /* Create the "working" file */
    let mut open_file = File::create(&temp_download_url)?;

    /* Download! */
    let mut easy = Easy::new();
    easy.url(url)?;

    easy.write_function(move |data| {
        Ok(open_file.write(data).unwrap())
    })?;

    easy.perform()?;

    /* At this time, the download is complete. Now let's move the file */
    rename(temp_download_url, destination)
}

fn set_wallpaper(path: &Path) -> std::io::Result<()> {
    let path = match path.to_str() {
        None => return Err(Error::new(ErrorKind::Other, "Could not convert path to string..")),
        Some(path) => path
    };

    /* If we are on Windows, do something special. Otherwise, just use feh */
    let result = if cfg!(target_os = "windows") {
        Command::new("reg")
            .arg("add")
            .arg("\"HKEY_CURRENT_USER\\Control Panel\\Desktop\"")
            .arg("/v")
            .arg("Wallpaper")
            .arg("/t")
            .arg("REG_SZ")
            .arg("/d")
            .arg(path)
            .arg("/f")
            .status()?;

        Command::new("RUNDLL32.EXE")
            .arg("user32.dll,UpdatePerUserSystemParameters")
            .status()?
    }
    else {
        Command::new("feh")
            .arg("--bg-fill")
            .arg(path)
            .status()?
    };

    if result.success() {
        Ok(())
    }
    else {
        Err(Error::new(ErrorKind::Other, "Setting wallpaper exited with unknown error"))
    }
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
    let imagedir = Path::new(matches.value_of("imagedir").unwrap());

    /* Step 1: Load all URLs from the URLs file */
    let urls = match load_urls_from_file(&urlfile) {
        Err(why) => panic!("Error loading URLS from file {} : {}", urlfile, why),
        Ok(urls) => urls
    };

    /* Step 2: Load files from imagedir as cache */
    let cache_hashes = match get_filenames_in_dir(&imagedir) {
        Err(why) => panic!("Error reading files in image dir {} : {}", imagedir.display(), why),
        Ok(cache) => cache
    };

    /* Step 3: Select a local image to display */
    let mut random = thread_rng();
    let local_image = random.choose(&cache_hashes);
    let remote_image = random.choose(&urls);

    /* Step 4: Decide whether we'll be trying to load a local image or URL */
    let should_local = random.gen_weighted_bool(3);

    /* Step 5: 
    If local image, set wallpaper to selected image 
    */
    if should_local && local_image.is_some() {
        set_wallpaper(&imagedir.join(&local_image.unwrap()).as_path()).unwrap();
    }
    else if remote_image.is_some() {
        let url_hash = get_url_hash(&remote_image.unwrap());
        let download_dest_buf = imagedir.join(&url_hash);
        let download_dest = download_dest_buf.as_path();
        if !download_dest.exists() {
            let download_result = download_remote_url(&remote_image.unwrap(), &download_dest);
            if download_result.is_err() && local_image.is_some() {
                set_wallpaper(&imagedir.join(&local_image.unwrap()).as_path()).unwrap();
            }
            else if download_result.is_ok() {
                set_wallpaper(&download_dest).unwrap();
            }
        }
        else {
            set_wallpaper(&download_dest).unwrap();
        }
    }
}