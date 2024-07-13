use std::path::PathBuf;

use anyhow::Result;
use bore_cli::{client::Client, server::Server};
use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    
    /// Optional path to file where secret for authentication is stored.
    #[arg(short, long)]
    filepath: Option<PathBuf>,

    /// Optional secret for authentication.
    #[arg(short, long, env = "BORE_SECRET", hide_env_values = true)]
    secret: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Starts a local proxy to the remote server.
    Local {
        /// The local port to expose.
        local_port: u16,

        /// The local host to expose.
        #[clap(short, long, value_name = "HOST", default_value = "localhost")]
        local_host: String,

        /// Address of the remote server to expose local ports to.
        #[clap(short, long, env = "BORE_SERVER")]
        to: String,

        /// Optional port on the remote server to select.
        #[clap(short, long, default_value_t = 0)]
        port: u16,



    },

    /// Runs the remote proxy server.
    Server {
        /// Minimum accepted TCP port number.
        #[clap(long, default_value_t = 1024)]
        min_port: u16,

        /// Maximum accepted TCP port number.
        #[clap(long, default_value_t = 65535)]
        max_port: u16,

    },
}

#[tokio::main]
async fn run(command: Command) -> Result<()> {
    let args = Args::parse();
    let filepath = args.filepath;
    let secret = args.secret;

    match command {
        Command::Local {
            local_host,
            local_port,
            to,
            port,
        } => {
            let client = Client::new(&local_host, local_port, &to, port, secret, filepath).await?;
            client.listen().await?;
        }
        Command::Server {
            min_port,
            max_port,
        } => {
            let port_range = min_port..=max_port;
            if port_range.is_empty() {
                Args::command()
                    .error(ErrorKind::InvalidValue, "port range is empty")
                    .exit();
            }
            Server::new(port_range, secret, filepath).await.listen().await?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    run(Args::parse().command)
}
