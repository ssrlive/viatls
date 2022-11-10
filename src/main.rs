use std::fs::File;
use viatls::{client, cmdopt, config, program_name, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = cmdopt::CmdOpt::parse_cmd();
    let is_server = opt.is_server();
    let (config, verbose) = match opt {
        cmdopt::CmdOpt::Server { config, verbose } => (config, verbose),
        cmdopt::CmdOpt::Client { config, verbose } => (config, verbose),
    };

    if let Err(_) = std::env::var("RUST_LOG") {
        let name = program_name();
        if verbose {
            std::env::set_var("RUST_LOG", format!("{}=trace", name));
        } else {
            std::env::set_var("RUST_LOG", format!("{}=warn", name));
        }
    };

    env_logger::init();

    let f = File::open(config)?;
    let mut config: config::Config = serde_json::from_reader(f)?;
    config.check_correctness()?;
    if is_server {
        if config.exist_server() {
            server::run_server(&config).await?;
        } else {
            anyhow::bail!("Config is not a server config");
        }
    } else if config.exist_client() {
        client::run_client(&config).await?;
    } else {
        anyhow::bail!("Config is not a client config");
    }

    Ok(())
}
