pub mod command;
pub mod error;
pub mod git;

use anyhow::{bail, Result};
use command::statistics::statistics;
use command::statistics_configuration::OutputFormat;
use log::{LevelFilter, Record};
use std::env;
use std::io::Write;
use std::str::FromStr;

type ExitCode = i32;

type LogFormatter = Box<
    dyn Fn(&mut env_logger::fmt::Formatter, &Record) -> Result<(), std::io::Error> + Send + Sync,
>;

fn main() {
    // Command line interface.
    let mut application = create_application();
    let mut exit_code: ExitCode = 0;
    if env::args().count() <= 1 {
        application.print_long_help().unwrap();
        println!();
    } else {
        let args = application.get_matches();

        // Initialize logger.
        initialize_logger(&args);

        // Run the application.
        exit_code = match run(&args) {
            Ok(exit_code) => exit_code,
            Err(e) => {
                eprintln!("{}", e);
                2
            }
        };
    }
    std::process::exit(exit_code);
}

/// Create the application command line interface.
fn create_application() -> clap::App<'static, 'static> {
    let bin_name = clap::crate_name!();

    clap::App::new(bin_name)
        .bin_name(bin_name)
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("verbosity")
                .long("verbosity")
                .help("Logging verbosity level")
                .takes_value(true)
                .possible_values(&["error", "warn", "info", "debug", "trace"])
                .required(false)
                .env("JIKYUU_LOG_LEVEL")
                .default_value("info"),
        )
        .arg(
            clap::Arg::with_name("log_format")
                .long("log-format")
                .help("Logging format")
                .takes_value(true)
                .possible_values(&["simple", "context"])
                .required(false)
                .default_value("simple"),
        )
        .subcommand(
            clap::SubCommand::with_name("completions")
                .about("Print shell completions")
                .arg(
                    clap::Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&["Bash", "Elvish", "Fish", "PowerShell", "Zsh"])
                        .case_insensitive(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("stats")
                .alias("statistics")
                .about("Print repository statistics")
                .arg(clap::Arg::with_name("max-commit-diff")
                     .long("max-commit-diff")
                     .short("d")
                     .help("Maximum difference in minutes between commits counted to one session")
                     .takes_value(true)
                     .value_name("MINUTES")
                     .required(false)
                     .default_value("120"))
                .arg(clap::Arg::with_name("first-commit-add")
                     .long("first-commit-add")
                     .short("a")
                     .help("How many minutes first commit of session should add to total")
                     .takes_value(true)
                     .value_name("MINUTES")
                     .required(false)
                     .default_value("30"))
                .arg(clap::Arg::with_name("since")
                     .long("since")
                     .short("s")
                     .help("Analyze data since certain date")
                     .takes_value(true)
                     .value_name("always|today|yesterday|thisweek|lastweek|YYYY-mm-dd")
                     .required(false)
                     .default_value("always"))
                .arg(clap::Arg::with_name("until")
                     .long("until")
                     .short("u")
                     .help("Analyze data until certain date")
                     .takes_value(true)
                     .value_name("always|today|yesterday|thisweek|lastweek|YYYY-mm-dd")
                     .required(false)
                     .default_value("always"))
                .arg(clap::Arg::with_name("email")
                     .long("email")
                     .short("e")
                     .help("Associate all commits that have a secondary email with a primary email")
                     .takes_value(true)
                     .multiple(true)
                     .number_of_values(1)
                     .value_name("OTHER_EMAIL=MAIN_EMAIL"))
                .arg(clap::Arg::with_name("merge-requests")
                     .long("merge-requests")
                     .short("m")
                     .help("Include merge requests into calculation"))
                .arg(clap::Arg::with_name("branch")
                     .long("branch")
                     .short("b")
                     .takes_value(true)
                     .help("Analyze only data on the specified branch"))
                .arg(clap::Arg::with_name("branch-type")
                     .long("branch-type")
                     .short("t")
                     .takes_value(true)
                     .value_name("local|remote")
                     .requires("branch")
                     .help("Type of branch that `branch` refers to. `local` means refs/heads/, `remote` means refs/remotes/."))
                .arg(clap::Arg::with_name("format")
                     .long("format")
                     .short("f")
                     .takes_value(true)
                     .possible_values(&OutputFormat::variants())
                     .case_insensitive(true)
                     .required(false)
                     .default_value("stdout"))
                .arg(clap::Arg::with_name("REPO_PATH")
                     .help("Root path of the Git repository to analyze.")
                     .required(true)
                     .default_value(".")
                     .index(1))
        )
}

/// Initializes the application logger.
fn initialize_logger(args: &clap::ArgMatches) {
    let args_log_level = args.value_of("verbosity").unwrap_or("error");
    let args_log_format = args.value_of("log_format").unwrap_or("simple");

    let log_level = LevelFilter::from_str(args_log_level).unwrap();
    let log_format: LogFormatter = match args_log_format {
        "context" => Box::new(|buf: &mut env_logger::fmt::Formatter, record: &Record| {
            writeln!(
                buf,
                "[{} {}] {}: {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
            .expect("Failed to write log message to buffer.");

            Ok(())
        }),
        _ => Box::new(|buffer: &mut env_logger::fmt::Formatter, record: &Record| {
            writeln!(buffer, "{}", record.args()).expect("Failed to write log message to buffer.");

            Ok(())
        }),
    };

    let env = env_logger::Env::default().filter("JIKYUU_LOG_LEVEL");

    env_logger::Builder::from_env(env)
        .filter_level(log_level)
        .format(log_format)
        .target(env_logger::Target::Stdout)
        .init();
}

/// Command to output completions of a specific type to STDOUT.
fn completions(args: &clap::ArgMatches) -> Result<ExitCode> {
    // Parse arguments.
    let args_completions = args.subcommand_matches("completions").unwrap();
    let completion_type = args_completions.value_of("type").unwrap();

    // Generate completion.
    if completion_type == "bash" {
        create_application().gen_completions_to(
            create_application().get_bin_name().unwrap(),
            clap::Shell::Bash,
            &mut std::io::stdout(),
        );
    } else if completion_type == "elvish" {
        create_application().gen_completions_to(
            create_application().get_bin_name().unwrap(),
            clap::Shell::Elvish,
            &mut std::io::stdout(),
        );
    } else if completion_type == "fish" {
        create_application().gen_completions_to(
            create_application().get_bin_name().unwrap(),
            clap::Shell::Fish,
            &mut std::io::stdout(),
        );
    } else if completion_type == "powershell" {
        create_application().gen_completions_to(
            create_application().get_bin_name().unwrap(),
            clap::Shell::PowerShell,
            &mut std::io::stdout(),
        );
    } else if completion_type == "zsh" {
        create_application().gen_completions_to(
            create_application().get_bin_name().unwrap(),
            clap::Shell::Zsh,
            &mut std::io::stdout(),
        );
    } else {
        bail!("Completion type '{}' not supported.", completion_type);
    }

    Ok(0)
}

/// Run application according to command line interface arguments.
fn run(args: &clap::ArgMatches) -> Result<ExitCode> {
    let mut exit_code: ExitCode = 0;

    if args.subcommand_matches("completions").is_some() {
        exit_code = completions(args)?;
    } else if args.subcommand_matches("stats").is_some() {
        exit_code = statistics(args)?;
    } else {
        create_application().print_long_help()?;
    }

    Ok(exit_code)
}
