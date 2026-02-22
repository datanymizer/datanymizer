use anyhow::Result;
use clap::Parser;

use app::App;
use options::Options;

mod app;
mod options;

fn main() -> Result<()> {
    let options = Options::parse();

    env_logger::init_from_env(env_logger::Env::default().filter_or(
        "RUST_LOG",
        match options.verbose {
            0 => "error",
            1 => "warn",
            2 => "info",
            3 => "debug",
            _ => "trace",
        },
    ));

    let app = App::from_options(options)?;
    app.run()
}
