mod reporter;

use color_eyre::eyre::Result;
// TODO use a color library with no runtime dependency like yansi
use deno_core::error::JsError;
use jwalk::WalkDir;
use rayon::prelude::*;
use rusty_v8 as v8;
use serde_yaml::Value::Sequence;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use reporter::Diagnostics;

// TODO get rid of this function. This is just a match wrapper around generate_final_file_to_test
pub fn generate_and_run(
    file_to_test: &PathBuf,
    include_contents: String,
    diagnostics: Arc<Diagnostics>,
) {
    match generate_final_file_to_test(file_to_test, include_contents) {
        Ok(file_to_run) => match spawn_v8_process(file_to_test, file_to_run) {
            None => reporter::pass(diagnostics, file_to_test),
            Some(e) => reporter::fail(diagnostics, file_to_test, e),
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
        let entry = entry?;
        if entry.file_type().is_file() {
            final_paths.push(entry.path());
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

pub fn get_contents(
    includes_map_r: evmap::ReadHandle<String, String>,
    includes_map_w: Arc<Mutex<evmap::WriteHandle<String, String>>>,
    includes_value: &serde_yaml::Value,
    include_path_root: &PathBuf,
) -> Result<String> {
    let mut includes: Vec<String> = vec!["assert".to_string(), "sta".to_string()];
    includes.append(&mut serde_yaml::from_value(includes_value.clone())?);
    let includes_clone = includes.clone();
    let mut w = includes_map_w.lock().unwrap();
    for include in includes {
        let include_clone = include.clone();
        if !(w.contains_key(&include)) {
            // read &include and store it in hashmap
            w.insert(
                include,
                fs::read_to_string(include_path_root.join(include_clone)).unwrap(),
            );
        }
    }
    w.refresh();
    let mut contents = String::new();
    for include in includes_clone {
        match includes_map_r.get(&include) {
            Some(includes_value) => {
                contents.push_str(&*includes_value.get_one().unwrap());
                contents.push('\n');
            }
            None => eprintln!("This should not have happened :|"),
        }
    }
    Ok(contents)
}

// TODO combine this and get_contents()
fn generate_final_file_to_test(file_to_test: &PathBuf, mut contents: String) -> Result<String> {
    let file_to_test_contents = fs::read_to_string(file_to_test)?;
    contents.push_str(&file_to_test_contents);
    Ok(contents)
}

pub fn spawn_v8_process(file: &PathBuf, js_source: String) -> Option<JsError> {
    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let source = v8::String::new(scope, &js_source).unwrap();

    let origin = v8::ScriptOrigin::new(
        v8::String::new(scope, file.to_str().unwrap())
            .unwrap()
            .into(),
        v8::Integer::new(scope, 0),
        v8::Integer::new(scope, 0),
        v8::Boolean::new(scope, false),
        v8::Integer::new(scope, 123),
        v8::String::new(scope, "").unwrap().into(),
        v8::Boolean::new(scope, true),
        v8::Boolean::new(scope, false),
        v8::Boolean::new(scope, false),
    );

    let tc_scope = &mut v8::TryCatch::new(scope);

    let script = match v8::Script::compile(tc_scope, source, Some(&origin)) {
        Some(script) => script,
        None => {
            let exception = tc_scope.exception().unwrap();
            return Some(JsError::from_v8_exception(tc_scope, exception));
        }
    };

    match script.run(tc_scope) {
        Some(_) => None,
        None => {
            assert!(tc_scope.has_caught());
            let exception = tc_scope.exception().unwrap();
            Some(JsError::from_v8_exception(tc_scope, exception))
        }
    }
}

pub fn run_all(test_path: PathBuf, include_path: PathBuf) -> Result<()> {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let (includes_map_r, includes_map_w) = evmap::new();
    let w = Arc::new(Mutex::new(includes_map_w));
    {
        let mut w = w.lock().unwrap();
        w.insert(
            "assert".to_string(),
            fs::read_to_string(include_path.join("assert.js"))?,
        );
        w.insert(
            "sta".to_string(),
            fs::read_to_string(include_path.join("sta.js"))?,
        );
        w.refresh();
    }

    let files_to_test = walk(test_path)?;
    let diagnostics = Arc::new(Diagnostics::new());
    files_to_test
        .into_par_iter()
        .for_each_with(includes_map_r, |includes_map_r, file| {
            let frontmatter = extract_frontmatter(&file);
            match frontmatter {
                Some(f) => match get_serde_value(&f) {
                    Ok(frontmatter_value) => match frontmatter_value.get("includes") {
                        Some(includes_value) => {
                            match get_contents(
                                includes_map_r.clone(),
                                w.clone(),
                                includes_value,
                                &include_path,
                            ) {
                                Ok(include_contents) => {
                                    generate_and_run(&file, include_contents, diagnostics.clone())
                                }
                                Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                            }
                        }
                        None => {
                            match get_contents(
                                includes_map_r.clone(),
                                w.clone(),
                                &Sequence([].to_vec()),
                                &include_path,
                            ) {
                                Ok(include_contents) => {
                                    generate_and_run(&file, include_contents, diagnostics.clone())
                                }
                                Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                            }
                        }
                    },
                    Err(e) => eprintln!(
                        "Could not get serde value from frontmatter in file {:?} | Err: {}",
                        file, e
                    ),
                },
                None => {
                    // FIXME This doesn't seem to work as intended
                    if file.ends_with("FIXTURE.js") {
                    } else {
                        diagnostics.invalid.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
    reporter::final_results(diagnostics);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::Diagnostics;
    use static_assertions::assert_impl_all;
    use std::marker::Send;
    use std::marker::Sync;

    #[test]
    fn test() {
        assert_impl_all!(Diagnostics: Send, Sync);
    }
}
