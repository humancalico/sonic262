use argh::FromArgs;
use color_eyre::Result;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::process::Command;
use tokio::task;
use walkdir::WalkDir;

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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Args = argh::from_env();

    let mut handles = Vec::new();

    let test_files = walk(&args.test262_dir)?;

    // Read harness/assert.js and harness/sta.js
    let assert: String = fs::read_to_string(args.test262_dir.join("harness/assert.js")).await?;
    let sta: String = fs::read_to_string(args.test262_dir.join("harness/sta.js")).await?;

    for file in test_files {
        let assert = assert.clone();
        let sta = sta.clone();
        let file = file.clone();
        let handle = task::spawn(async move {
            let res = test_file(&file, assert, sta).await;
            if let Err(e) = res {
                println!("Encountered an error while testing file: {}", e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

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

async fn test_file(path: &Path, assert: String, sta: String) -> Result<()> {
    let mut final_file_contents = String::new();
    final_file_contents.push_str(&assert);
    final_file_contents.push_str(&sta);
    let test_file: String = fs::read_to_string(path).await?;
    final_file_contents.push_str(&test_file);
    let temp_file = NamedTempFile::new()?;
    // referring to a temporary file's path may not be secure. Find better alternative
    fs::write(temp_file.path(), final_file_contents).await?;
    let path_os_str: &OsStr = OsStr::new(temp_file.path());
    let exit_status = Command::new("node").arg(path_os_str).status().await?;
    if exit_status.success() {
        println!("PASS");
    } else {
        println!("FAIL");
    }
    println!("{} => {:?}", path.display(), std::thread::current().id());
    Ok(())
}
