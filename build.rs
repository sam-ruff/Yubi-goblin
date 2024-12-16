use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let _ = embed_resource::compile("resource.rc", embed_resource::NONE);
    let out_dir = env::current_dir()?;
    let dest_path = Path::new(&out_dir).join("static_files.rs");
    let mut f = File::create(&dest_path)?;
    let static_files_path = Path::new(&out_dir).join("static");
    let static_files_dir = static_files_path.as_path();
    let mut files = Vec::new();

    // Recursively list all files
    visit_dirs(static_files_dir, &mut files)?;

    writeln!(f, "use std::collections::HashMap;")?;
    writeln!(f, "pub fn static_files() -> HashMap<&'static str, &'static [u8]> {{")?;
    writeln!(f, "    let mut files = HashMap::new();")?;

    for file in files {
        let path = file.strip_prefix(static_files_dir).unwrap().to_str().unwrap().replace("\\", "/");
        // Tell Cargo to re-run this script if the file changes
        println!("cargo:rerun-if-changed={}", file.to_str().unwrap());

        writeln!(
            f,
            "    files.insert(r#\"{}\"#, include_bytes!(r#\"{}\"#).as_ref());",
            path, file.display()
        )?;
    }

    writeln!(f, "    files")?;
    writeln!(f, "}}")?;

    Ok(())
}

// Recursively visits directories and adds all file paths to the provided vector
fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // If it's a directory, recursively visit it
                visit_dirs(&path, files)?;
            } else {
                // Add file paths to the vector
                files.push(path);
            }
        }
    }
    Ok(())
}
