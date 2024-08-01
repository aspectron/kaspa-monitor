use kaspa_utils::networking::ContextualNetAddress;

#[derive(Debug)]
pub struct Args {
    pub verbose: bool,
    pub trace: bool,
    pub debug: bool,
    pub rpc_listen: ContextualNetAddress,
}

impl Args {
    pub fn parse() -> Args {
        #[allow(unused)]
        use clap::{arg, command, Arg, Command};

        let cmd = Command::new("sparkled")
            .about(format!(
                "kaspa cluster monitor v{}-{}",
                crate::VERSION,
                crate::GIT_DESCRIBE,
                // kaspa_wallet_core::version()
            ))
            .arg(arg!(--version "Display software version"))
            .arg(arg!(--verbose "Enable verbose mode"))
            .arg(arg!(--trace "Enable trace log level"))
            .arg(arg!(--debug "Enable debug mode"))
            .arg(arg!(--http "Enable HTTP Server"))
            .arg(
                Arg::new("rpc-listen")
                    .long("rpc-listen")
                    .value_name("ip[:port]")
                    .num_args(0..=1)
                    .require_equals(true)
                    .value_parser(clap::value_parser!(ContextualNetAddress))
                    .help(
                        "Interface:port to listen for wRPC connections (default: 127.0.0.1:6969).",
                    ),
            )
            .arg(
                Arg::new("node-rpc")
                    .long("node-rpc")
                    .value_name("ws://address[:port] or wss://address[:port]")
                    .num_args(0..=1)
                    .require_equals(true)
                    .help("wRPC URL of the node (disables resolver)."),
            );

        let matches = cmd.get_matches();

        let trace = matches.get_one::<bool>("trace").cloned().unwrap_or(false);
        let debug = matches.get_one::<bool>("debug").cloned().unwrap_or(false);
        let verbose = matches.get_one::<bool>("verbose").cloned().unwrap_or(false);

        let rpc_listen = matches
            .get_one::<ContextualNetAddress>("rpc-listen")
            .cloned()
            .unwrap_or("127.0.0.1:6969".parse().unwrap());

        if matches.get_one::<bool>("version").cloned().unwrap_or(false) {
            println!("v{}-{}", crate::VERSION, crate::GIT_DESCRIBE);
            std::process::exit(0);
        } else {
            Args {
                trace,
                debug,
                verbose,
                rpc_listen,
            }
        }
    }
}

impl AsRef<Args> for Args {
    fn as_ref(&self) -> &Args {
        self
    }
}

impl From<&Args> for kaspa_monitor_nexus::args::Args {
    fn from(args: &Args) -> Self {
        kaspa_monitor_nexus::args::Args {
            verbose: args.verbose,
            trace: args.trace,
            debug: args.debug,
        }
    }
}
