use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "roxy")]
#[command(about = "HTTP/HTTPS intercepting proxy")]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Listening host
    #[arg(short, long, default_value = "127.0.0.1")]
    pub host: String,

    /// Listening port
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Enable HTTPS passthrough for all hosts
    #[arg(long)] pub passthrough: bool,

    /// Enable HTTPS passthrough for a specific host
    #[arg(long = "passthrough-host", num_args=1..)] 
    pub passthrough_hosts: Vec<String>,

    /// Enable HTTPS interception for a specific host (usualy combined with --passthrough)
    #[arg(long = "intercept-https", num_args=1..)]
    pub mitm_hosts: Vec<String>,

    /// Print help
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,
}


#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub https: HttpsConfig,
}

#[derive(Clone)]
pub struct HttpsConfig {
    pub passthrough: bool,
    pub passthrough_hosts: Vec<String>,
    pub intercept_hosts: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            https: HttpsConfig {
                passthrough: false,
                passthrough_hosts: Vec::new(),
                intercept_hosts: Vec::new(),
            },
        }
    }
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            host: args.host,
            port: args.port,
            https: HttpsConfig {
                passthrough: args.passthrough,
                passthrough_hosts: args.passthrough_hosts,
                intercept_hosts: args.mitm_hosts,
            },
        }
    }
}
