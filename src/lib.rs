use color_eyre::eyre::Result;
// TODO use a color library with no runtime dependency like yansi
use colored::Colorize;
use rayon::prelude::*;
use serde_yaml::Value::Sequence;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tempfile::NamedTempFile;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Diagnostics {
    total_files: AtomicU32,
    run: AtomicU32,
    passed: AtomicU32,
    failed: AtomicU32,
    no_frontmatter: AtomicU32,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            total_files: AtomicU32::new(0),
            run: AtomicU32::new(0),
            passed: AtomicU32::new(0),
            failed: AtomicU32::new(0),
            no_frontmatter: AtomicU32::new(0),
        }
    }
}

pub fn generate_and_run(
    file_to_test: &PathBuf,
    files_to_add: Vec<PathBuf>,
    diagnostics: Arc<Diagnostics>,
    must_include: Arc<String>
) {
    match generate_final_file_to_test(file_to_test, files_to_add, must_include) {
        Ok(file_to_run) => match spawn_node_process(file_to_run) {
            Ok(exit_status) => {
                if exit_status.success() {
                    diagnostics.run.fetch_add(1, Ordering::Relaxed);
                    diagnostics.passed.fetch_add(1, Ordering::Relaxed);
                    println!("{} {:?}", "Great Success".green(), file_to_test);
                } else {
                    diagnostics.run.fetch_add(1, Ordering::Relaxed);
                    diagnostics.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("{} {:?}", "FAIL".red(), file_to_test);
                }
            }
            Err(e) => eprintln!("Failed to execute the file_to_test | Error: {:?}", e),
        },
        Err(e) => eprintln!(
            "Couldn't generate file: {:?} to test | Err: {}",
            file_to_test, e
        ),
    }
}

fn walk(root_path: PathBuf) -> Result<Vec<PathBuf>> {
    let mut final_paths: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(root_path) {
        // FIXME possible unecessary clone
        let entry_clone = entry?.clone();
        if entry_clone.file_type().is_file() {
            final_paths.push(entry_clone.into_path());
        } else {
        }
    }
    Ok(final_paths)
}

pub fn extract_frontmatter(file_to_test: &PathBuf) -> Option<String> {
    // FIXME remove unwrap
    let file_contents = fs::read_to_string(file_to_test).unwrap();
    let yaml_start = file_contents.find("/*---");
    if let Some(start) = yaml_start {
        let yaml_end = file_contents.find("---*/");
        if let Some(end) = yaml_end {
            // TODO remove unwrap here
            Some(file_contents.get(start + 5..end).unwrap().to_string())
        } else {
            eprintln!("This file has an invalid frontmatter");
            None
        }
    } else {
        eprintln!("frontmatter not found in file: {:?}", file_to_test);
        None
    }
}

pub fn get_serde_value(frontmatter_str: &str) -> serde_yaml::Result<serde_yaml::Value> {
    serde_yaml::from_str(frontmatter_str)
}

pub fn get_include_paths(
    includes_value: &serde_yaml::Value,
    include_path_root: &PathBuf,
) -> serde_yaml::Result<Vec<PathBuf>> {
    let includes: Vec<String> = serde_yaml::from_value(includes_value.clone())?;
    let mut include_paths: Vec<PathBuf> = vec![];
    includes.into_iter().for_each(|include| {
        include_paths.push(include_path_root.join(include));
    });
    Ok(include_paths)
}

fn generate_final_file_to_test(
    file_to_test: &PathBuf,
    files_to_add: Vec<PathBuf>,
    must_include: Arc<String>
) -> Result<NamedTempFile> {
    let mut contents = String::new();
    contents.push_str(&*must_include);
    for file in files_to_add {
        let file_contents = fs::read_to_string(file)?;
        contents.push_str(&file_contents);
    }
    let file_to_test_contents = fs::read_to_string(file_to_test)?;
    contents.push_str(&file_to_test_contents);
    let mut file = tempfile::Builder::new().suffix(".js").tempfile()?;
    writeln!(file, "{}", contents)?;
    Ok(file)
}

pub fn spawn_node_process(file: NamedTempFile) -> std::io::Result<ExitStatus> {
    // TODO .status() waits for the command to execute
    // FIXME currently shows all the errors if node is not able to run the file
    Command::new("node").arg(file.path()).status()
}

pub fn run_all(test_path: PathBuf, include_path: PathBuf) -> Result<()> {
    let assert = fs::read_to_string(include_path.join("assert.js"))?;
    let sta = fs::read_to_string(include_path.join("sta.js"))?;
    let must_include = format!("{}\n{}\n", assert, sta);
    let must_include = Arc::new(must_include);

    let files_to_test = walk(test_path)?;
    let diagnostics = Arc::new(Diagnostics::new());
    files_to_test.into_par_iter().for_each(|file| {
        let frontmatter = extract_frontmatter(&file);
        match frontmatter {
            Some(f) => match get_serde_value(&f) {
                Ok(frontmatter_value) => match frontmatter_value.get("includes") {
                    Some(includes_value) => {
                        match get_include_paths(includes_value, &include_path) {
                            Ok(include_files) => {
                                generate_and_run(&file, include_files, diagnostics.clone(), must_include.clone())
                            }
                            Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                        }
                    }
                    None => match get_include_paths(&Sequence([].to_vec()), &include_path) {
                        Ok(include_files) => {
                            generate_and_run(&file, include_files, diagnostics.clone(), must_include.clone())
                        }
                        Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                    },
                },
                Err(e) => eprintln!("Could not get serde value from frontmatter | Err: {}", e),
            },
            None => {
                diagnostics.no_frontmatter.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
    println!(
        "TOTAL: {}, FAILED: {}, PASSED: {}, NO FRONTMATTER: {}",
        diagnostics.run.load(Ordering::Relaxed).to_string().yellow(),
        diagnostics.failed.load(Ordering::Relaxed).to_string().red(),
        diagnostics
            .passed
            .load(Ordering::Relaxed)
            .to_string()
            .green(),
        diagnostics
            .no_frontmatter
            .load(Ordering::Relaxed)
            .to_string()
            .cyan(),
    );
    Ok(())
}

//fn store_includes_in_hashmap() -> Result<()> {}

#[cfg(test)]
mod test {
    use super::Diagnostics;
    use static_assertions::assert_impl_all;

    #[test]
    fn test() {
        assert_impl_all!(Diagnostics: std::marker::Send, std::marker::Sync);
    }
}
