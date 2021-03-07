use anyhow::Result;
use structopt::StructOpt;

mod app;
mod options;

use app::App;
use options::Options;

#[async_std::main]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let app = App::from_options(options)?;
    app.run().await
}
