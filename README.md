wallers
=======
A Rust implementation of [wallepy](https://github.com/brandonio21/wallepy). 
Define wallpapers as a list of URLs which wallers will keep track of and randomly
select a wallpaper to download and set.

Dependencies
------------
* Rust

For Linux Users,
* feh

Workflow
--------
wallers will select either a locally downloaded image or a URL to set as the wallpaper.
If wallers chooses a remote URL which hasn't already been downloaded, it will download the image and set the wallpaper. 
Images are only downloaded once.

Thus, there are two ways to influence the wallpapers that wallers chooses from:
1. Insert a remote URL into the URL list
2. Insert an image into the images folder

URL List
--------
By default, the URL list is found at `$(HOME)/.config/wallers/urls.txt` and is
comprised of newline delineated URLs. As an example,

```
<URL1>/wallpaper.png
<URL1>/wallpaper.jpg
<URL2>/wallpaper.png
```

Download Format
---------------
Downloaded images are written directly to disk and given a filename
corresponding to the hash of their URL. This allows images to only
be downloaded once.

Options
-------
For all users, two options are pertinent: `--urlfile` and `--imagedir`, which specify
where the URL list can be found and where downloaded images should be stored,
respectively.

For Windows users, `--force32bit` might be useful if operation without the flag
changes the wallpaper to all black.

For Linux users, the wallpaper is set using `feh`. Thus, `--fehpath` can be used
to specify the location of the `feh` executable if it is not in `$PATH`.