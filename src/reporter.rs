use colored::Colorize;
use deno_core::error::JsError;
use std::path::PathBuf;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub total_files: AtomicU32,
    pub run: AtomicU32,
    pub passed: AtomicU32,
    pub failed: AtomicU32,
    pub invalid: AtomicU32,
}

// This is to be called when a test is successful
pub fn pass(diagnostics: Arc<Diagnostics>, file_to_test: &PathBuf) {
    diagnostics.run.fetch_add(1, Ordering::Relaxed);
    diagnostics.passed.fetch_add(1, Ordering::Relaxed);
    println!("{} {:?}", "Great Success".green(), file_to_test);
}

// This is to be called when a test fails
pub fn fail(diagnostics: Arc<Diagnostics>, file_to_test: &PathBuf, e: JsError) {
    diagnostics.run.fetch_add(1, Ordering::Relaxed);
    diagnostics.failed.fetch_add(1, Ordering::Relaxed);
    eprintln!("{} {:?}", "FAIL".red(), file_to_test);
    eprintln!("{}", e);
}

// To print final results
pub fn final_results(diagnostics: Arc<Diagnostics>) {
    println!(
        "TOTAL: {}, FAILED: {}, PASSED: {}, INVALID: {}",
        diagnostics.run.load(Ordering::Relaxed).to_string().yellow(),
        diagnostics.failed.load(Ordering::Relaxed).to_string().red(),
        diagnostics
            .passed
            .load(Ordering::Relaxed)
            .to_string()
            .green(),
        diagnostics
            .invalid
            .load(Ordering::Relaxed)
            .to_string()
            .cyan(),
    );
}
