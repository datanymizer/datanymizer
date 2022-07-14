use anyhow::Result;
use structopt::StructOpt;

use app::App;
use options::Options;

mod app;
mod options;

fn main() -> Result<()> {
    let options = Options::from_args();
    env_logger::init();

    let app = App::from_options(options)?;
    app.run()
}
