/* Use clap for arg parsing */
extern crate clap;
extern crate rand;
extern crate curl;
extern crate easy_hash;
#[cfg(windows)] extern crate user32;

use std::io::{Read, Error, ErrorKind, Write};
use std::fs::{create_dir_all, rename, read_dir};
use std::fs::File;
use clap::{Arg, App};
use std::path::{Path, PathBuf};
use rand::{thread_rng, Rng};
use curl::easy::Easy;
use easy_hash::{Sha256, Hasher, HashResult};

#[cfg(windows)] use std::ffi::CString;
#[cfg(windows)] use std::os::raw::c_void;
#[cfg(not(windows))] use std::process::Command;

fn load_urls_from_file(path: &Path) -> std::io::Result<Vec<String>> {
    /* First, we will open the path given to us */
    let mut file = match File::open(path) {
        Err(why) => return Err(why),
        Ok(file) => file
    };

    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let mut url_list = Vec::new();
    for url in s.lines() {
        url_list.push(url.to_owned());
    }

    Ok(url_list)
}

fn get_filenames_in_dir(path: &Path) -> std::io::Result<Vec<String>> {
    create_dir_all(path)?;

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

fn get_default_config_path(filename: &Path, is_dir: bool) -> Option<PathBuf> {
    let home_dir = std::env::home_dir();
    match home_dir {
        None => None,
        Some(mut dir) => {
            dir.push(".config");
            dir.push("wallers");

            if !is_dir {
                if let Err(_) = create_dir_all(dir.as_path()) {
                    return None;
                }
            }

            dir.push(filename);

            if is_dir {
                if let Err(_) = create_dir_all(dir.as_path()) {
                    return None;
                }
            }

            match dir.as_path().exists() {
                false => None,
                true => Some(dir)
            }
        }
    }
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

#[cfg(windows)]
fn os_set_wallpaper(path: String, force_32bit: bool, _: &str) -> std::io::Result<()> {
    let wallpaper_func = if force_32bit || cfg!(target_pointer_width = "32") {
        user32::SystemParametersInfoA
    }
    else {
        user32::SystemParametersInfoW
    };


    let path_ptr = CString::new(path).unwrap();
    let path_ptr_c = path_ptr.into_raw();
    let result = unsafe {
        match path_ptr_c.is_null() {
            false => wallpaper_func(20, 0, path_ptr_c as *mut c_void, 0),
            true => 0
        }
    };

    unsafe {
        CString::from_raw(path_ptr_c)
    };

    match result {
        0 => Err(Error::new(ErrorKind::Other, "Failed to set Windows wallpaper")),
        _ => Ok(())
    }
}

#[cfg(not(windows))]
fn os_set_wallpaper(path: String, _: bool, feh_path: &str) -> std::io::Result<()> {
    let result = Command::new(feh_path)
        .arg("--bg-fill")
        .arg(path)
        .status()?.success();

    match result {
        true => Ok(()),
        false => Err(Error::new(ErrorKind::Other, "Failed to set non-Windows wallpaper"))
    }
}

fn set_wallpaper(path: &Path, force_32bit: bool, feh_path: &str) -> std::io::Result<()> {
    let path_str = match path.to_str() {
        None => return Err(Error::new(ErrorKind::Other, "Could not convert path to string..")),
        Some(path) => path.to_string()
    };

    os_set_wallpaper(path_str, force_32bit, feh_path)
}

fn main() {
    let default_urlfile_path = get_default_config_path(Path::new("urls.txt"), false);
    let default_imagedir_path = get_default_config_path(Path::new("images"), true);

    let matches = App::new("wallers")
        .version("0.1.2")
        .about("A wallpaper setter and getter")
        .author("brandonio21")
        .arg(Arg::with_name("urlfile")
            .short("u")
            .long("urlfile")
            .value_name("URLFILE")
            .help("File which contains the list of URLs")
            .takes_value(true)
            .required(default_urlfile_path.is_none()))
        .arg(Arg::with_name("imagedir")
            .short("d")
            .long("imagedir")
            .value_name("IMAGEDIR")
            .help("Path where downloaded images will be stored")
            .takes_value(true)
            .required(default_imagedir_path.is_none()))
        .arg(Arg::with_name("force32bit")
            .short("A")
            .long("force32bit")
            .help("Force 32-bit operation on Windows systems"))
        .arg(Arg::with_name("fehpath")
            .short("f")
            .long("fehpath")
            .value_name("FEHPATH")
            .help("Path to the feh executable (default: feh)")
            .takes_value(true))
        .get_matches();

    let urlfile = match matches.value_of("urlfile") {
        Some(path) => Path::new(path).to_path_buf(),
        None => default_urlfile_path.unwrap()
    };

    let imagedir = match matches.value_of("imagedir") {
        Some(path) => Path::new(path).to_path_buf(),
        None => default_imagedir_path.unwrap()
    };

    let fehpath = match matches.value_of("fehpath") {
        Some(path) => path,
        None => "feh"
    };

    let force32bit = matches.is_present("force32bit");

    /* Step 1: Load all URLs from the URLs file */
    let urls = match load_urls_from_file(urlfile.as_path()) {
        Err(why) => panic!("Error loading URLS from file {} : {}", urlfile.to_str().unwrap(), why),
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
        set_wallpaper(&imagedir.join(&local_image.unwrap()).as_path(), force32bit, fehpath).unwrap();
    }
    else if remote_image.is_some() {
        let url_hash = get_url_hash(&remote_image.unwrap());
        let download_dest_buf = imagedir.join(&url_hash);
        let download_dest = download_dest_buf.as_path();
        if !download_dest.exists() {
            let download_result = download_remote_url(&remote_image.unwrap(), &download_dest);
            if download_result.is_err() && local_image.is_some() {
                set_wallpaper(&imagedir.join(&local_image.unwrap()).as_path(), force32bit, fehpath).unwrap();
            }
            else if download_result.is_ok() {
                set_wallpaper(&download_dest, force32bit, fehpath).unwrap();
            }
        }
        else {
            set_wallpaper(&download_dest, force32bit, fehpath).unwrap();
        }
    }
}