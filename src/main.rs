use argh::FromArgs;
use color_eyre::Result;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use rayon::prelude::*;

/// Fast experimental harness for test262
#[derive(FromArgs, Clone)]
struct Args {
    //#[argh(positional)]
    //includes_dir: PathBuf,
    //#[argh(positional)]
    //test_files: Vec<PathBuf>,
    //#[argh(positional)]
    //host_path: Option<PathBuf>,
    //#[argh(positional)]
    //host_args: Option<String>,
    //#[argh(positional)]
    //threads: Option<usize>,
    //#[argh(positional)]
    //features: Option<Vec<String>>,
    #[argh(positional)]
    test262_dir: PathBuf,
    //#[argh(positional)]
    //temp_dir: Option<PathBuf>,
    //#[argh(positional)]
    //prelude: Option<PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Args = argh::from_env();

    let test_files = walk(&args.test262_dir)?;

    // Read harness/assert.js and harness/sta.js
    let assert: String = fs::read_to_string(args.test262_dir.join("harness/assert.js"))?;
    let sta: String = fs::read_to_string(args.test262_dir.join("harness/sta.js"))?;

    test_files.par_iter().for_each(|file| {
        let assert = assert.clone();
        let sta = sta.clone();
        let file = file.clone();
        let res = test_file(&file, assert, sta);
        if let Err(e) = res {
            println!("Encountered an error while testing file: {}", e);
        }
    });

    Ok(())
}

fn walk(path: &PathBuf) -> walkdir::Result<Vec<PathBuf>> {
    let mut final_paths: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(path) {
        // FIXME possible unecessary clone
        let entry_clone = entry?.clone();
        if entry_clone.file_type().is_file() {
            final_paths.push(entry_clone.into_path());
        } else {
        }
    }
    Ok(final_paths)
}

fn test_file(path: &Path, assert: String, sta: String) -> Result<()> {
    let mut final_file_contents = String::new();
    final_file_contents.push_str(&assert);
    final_file_contents.push_str(&sta);
    let test_file: String = fs::read_to_string(path)?;
    final_file_contents.push_str(&test_file);
    let temp_file = NamedTempFile::new()?;
    // referring to a temporary file's path may not be secure. Find better alternative
    fs::write(temp_file.path(), final_file_contents)?;
    let path_os_str: &OsStr = OsStr::new(temp_file.path());
    let exit_status = Command::new("node").arg(path_os_str).status()?;
    if exit_status.success() {
        println!("PASS");
    } else {
        println!("FAIL");
    }
    println!("{} => {:?}", path.display(), std::thread::current().id());
    Ok(())
}
