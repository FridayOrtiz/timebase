use clap::{crate_authors, crate_description, crate_version, App, Arg, SubCommand};
use slog::debug;
use timebase::LOGGER;
use timebase::{client, dmz, server};

fn main() {
    let matches = App::new("Timebase")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("client")
                .about("receive information from the server")
                .arg(
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .help("the interface to intercept and modify communications on")
                        .takes_value(true)
                        .required(true)
                        .value_name("INTERFACE NAME"),
                ),
        )
        .subcommand(
            SubCommand::with_name("server")
                .about("send information to the client")
                .arg(
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .help("the interface to intercept and modify communications on")
                        .takes_value(true)
                        .required(true)
                        .value_name("INTERFACE NAME"),
                ),
        )
        .subcommand(
            SubCommand::with_name("dmz")
                .about("send information to the server")
                .arg(
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .help("the interface to intercept and modify communications on")
                        .takes_value(true)
                        .required(true)
                        .value_name("INTERFACE NAME"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        debug!(LOGGER, "Starting timebase server.");
        let interface = matches.value_of("interface").unwrap();
        server::load_filter(interface).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("client") {
        let interface = matches.value_of("interface").unwrap();
        client::run_client(interface);
    } else if let Some(matches) = matches.subcommand_matches("dmz") {
        let interface = matches.value_of("interface").unwrap();
        dmz::run_dmz(interface).unwrap();
    } else {
        println!("Please specify `client`, `dmz`, or `server`.");
    }
}
