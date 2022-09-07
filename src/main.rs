use std::env;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("need 2 params by MD_PATHS and SRC_DIR_PATHS");
    }

    let cur_path_buf = env::current_dir().unwrap();
    let cur_path = cur_path_buf.to_str().unwrap();

    let arg_to_full_paths = |arg: &str|
        arg.split(",")
            .map(|path| path.trim())
            .map(|path| fill_path_to_full(&cur_path, path))
            .collect::<Vec<_>>();

    let md_paths = arg_to_full_paths(&args[1]);
    let dir_paths = arg_to_full_paths(&args[2]);

    // println!("md_paths: {:?}, dir_paths: {:?}", md_paths, dir_paths);
    local_unused_pic_clean(md_paths, dir_paths)
}

pub fn local_unused_pic_clean(md_paths: Vec<String>, src_paths: Vec<String>) {
    let local_file_paths = md_paths.iter()
        .flat_map(|md_path| local_pics(md_path))
        .collect::<Vec<_>>();

    src_paths.iter()
        .flat_map(|dir_path| list_dir_file_paths(dir_path))
        .filter(|exist_file_path| !local_file_paths.contains(exist_file_path))
        .for_each(|exist_file_path| move_to_trash(&exist_file_path));
}

pub fn move_to_trash(file_path: &str) {
    // println!("move local image to movable dir: {}", &file_path);

    let file_path_obj = Path::new(file_path);
    let trash_path_buf = file_path_obj.parent().unwrap().to_path_buf().join("movable");
    let trash_path_obj = trash_path_buf.as_path();
    if !trash_path_obj.exists() {
        std::fs::create_dir(trash_path_obj)
            .expect(format!("create dir failed: {}", trash_path_obj.clone().to_str().unwrap()).as_str());
    }

    let file_name = file_path_obj.file_name().unwrap().to_str().unwrap();
    let new_file_path_buf = trash_path_obj.join(file_name);
    let new_file_path_obj = new_file_path_buf.as_path();
    let new_file_path = new_file_path_obj.to_str().unwrap();
    // println!("file mv: {} => {}", path_str, new_file_path);

    let r = std::fs::rename(file_path, new_file_path_obj);
    match r {
        Ok(_) => {
            // println!("file mv success: {} => {}", file_path, new_file_path);
        }
        Err(e) => {
            println!("file mv failed: {} => {}, {:?}", file_path, new_file_path, e);
        }
    }
}

pub fn list_dir_file_paths(dir_path: &str) -> Vec<String> {
    let dir = std::fs::read_dir(dir_path).unwrap();
    let mut dir_file_paths = Vec::new();
    for dir_entry_r in dir {
        let dir_entry = dir_entry_r.unwrap();
        let path_buf = dir_entry.path();
        if path_buf.is_file() {
            let path = path_buf.to_str().unwrap();
            dir_file_paths.push(String::from(path));
        }
    }
    dir_file_paths
}

pub fn local_pics(md_path: &str) -> Vec<String> {
    // println!("find md local image: {}", &md_path);
    let file = File::open(md_path).unwrap();
    let buf_reader = BufReader::new(file);

    let parent_path_obj = Path::new(md_path).parent().unwrap();
    let parent_path = parent_path_obj.to_str().unwrap();

    buf_reader.lines()
        .filter(|line_r| line_r.is_ok())
        .flat_map(|line_r| get_local_links(line_r.unwrap().as_str()))
        .map(|line| fill_path_to_full(parent_path, &line))
        .collect::<Vec<_>>()
}

pub fn fill_path_to_full(parent_path: &str, file_path: &str) -> String {
    if file_path.starts_with("/") || file_path.contains(":") {
        return String::from(file_path);
    }

    let mut parent_path_obj = Path::new(parent_path);
    let mut file_path = file_path;

    if file_path.starts_with("./") {
        file_path = &file_path[2..file_path.len() - 1] // remove prefix "./"
    }

    loop {
        if file_path.starts_with("../") {
            file_path = &file_path[3..file_path.len() - 1];  // remove prefix "../"
            parent_path_obj = parent_path_obj.parent().unwrap()
        } else {
            break;
        }
    }

    let full_path_buf = parent_path_obj.to_path_buf().join(Path::new(file_path));
    String::from(full_path_buf.as_path().to_str().unwrap())
}

pub fn get_local_links(line: &str) -> Vec<String> {
    let md_img_re = Regex::new("!\\[(?P<alt>.*?)]\\((?P<src>[^ ]*) *\"?(?P<title>.*?)\"??\\)").unwrap();
    let tag_img_re = Regex::new("<img.*?src=[\'\"](?P<src>.*?)[\'\"].*?>").unwrap();

    md_img_re.captures_iter(line)
        .chain(tag_img_re.captures_iter(line))
        .map(|captures| String::from(captures.name("src").unwrap().as_str()))
        .filter(|src| !is_remote_link(src))
        .collect::<Vec<_>>()
}

pub fn is_remote_link(link: &str) -> bool {
    let re = Regex::new("((http(s?))|(ftp))://.*").unwrap();
    re.is_match(link)
}
