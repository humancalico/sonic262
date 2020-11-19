use clap::Clap;
use color_eyre::eyre::Result;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(
    name = "sonic262",
    version = "0.1.0",
    about = "A harness for test262",
    author = "Ujjwal Sharma <ryzokuken@disroot.org>"
)]
struct Opts {
    // #[clap(long)]
    // root_path: Option<PathBuf>,
    #[clap(long)]
    test_path: PathBuf,
    #[clap(long)]
    include_path: PathBuf,
    // #[clap(long)]
    // files_to_test: Option<Vec<PathBuf>>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Opts::parse();
    // let files_to_test = args.files_to_test.unwrap();
    let include_path = args.include_path;
    let test_path = args.test_path;

    sonic262::run_all(test_path, include_path)
}
