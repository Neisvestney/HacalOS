use inithfs::InitHFsRoot;
use std::path::Path;
use std::{env, fs};
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let canonical_path = fs::canonicalize(directory_path).unwrap();

    let mut files = Vec::new();

    for entry in WalkDir::new(&canonical_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|metadata| metadata.file_type().is_file())
    {
        let path = entry.path();
        let contents = fs::read(path).unwrap();
        let relative_path = path.strip_prefix(&canonical_path).unwrap().to_str().unwrap().to_string();
        println!("{}", relative_path);
        files.push((relative_path, contents));
    }

    {
        let mut inithfs_root = InitHFsRoot::new();
        for (relative_path, contents) in &files {
            inithfs_root.add_file(relative_path, contents).unwrap();
        }
        let bytes = inithfs_root.to_bytes().unwrap();
        println!("Writing {}", output_path.display());
        fs::write(output_path, bytes).unwrap();
    }
}

