use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use std::{collections::HashMap, fs::File, io::Read, time::Duration};

pub struct BalanceVerificationConfig {
    pub redis_connection: String,
    pub circuit_level: u64,
    pub stop_after: u64,
    pub lease_for: u64,
    pub time_to_run: Option<Duration>,
    pub preserve_intermediary_proofs: bool,
    pub protocol: Option<String>,
}

pub fn parse_balance_verification_command_line_options(
    matches: &ArgMatches,
) -> BalanceVerificationConfig {
    let run_for_input = matches.value_of("run_for_minutes").unwrap();

    let time_to_run: Option<Duration> = match run_for_input {
        "infinity" => None,
        minutes => {
            let mins = minutes.parse::<u64>().expect("Failed to parse minutes");
            Some(Duration::from_secs(mins * 60))
        }
    };

    BalanceVerificationConfig {
        redis_connection: matches.value_of("redis_connection").unwrap().to_string(),
        circuit_level: matches
            .value_of("circuit_level")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        stop_after: matches
            .value_of("stop_after")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        lease_for: matches
            .value_of("lease_for")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        time_to_run,
        preserve_intermediary_proofs: matches.get_flag("preserve_intermediary_proofs"),
        protocol: match matches.value_of("protocol") {
            None => None,
            Some(protocol) => Some(protocol.to_owned()),
        },
    }
}

pub struct CommandLineOptionsBuilder<'a> {
    pub command: Command<'a>,
}

impl<'a> CommandLineOptionsBuilder<'a> {
    pub fn new(name: &str) -> Self {
        Self {
            command: Command::new(name),
        }
    }

    pub fn arg(self, a: Arg<'a>) -> Self {
        Self {
            command: self.command.arg(a),
        }
    }

    pub fn with_balance_verification_options(self) -> Self {
        let command = self
            .command
            .arg(
                Arg::with_name("circuit_level")
                    .short('l')
                    .long("level")
                    .value_name("LEVEL")
                    .help("Sets the circuit level")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("run_for_minutes")
                    .long("run-for")
                    .value_name("Run for X minutes")
                    .takes_value(true)
                    .default_value("infinity"),
            )
            .arg(
                Arg::with_name("preserve_intermediary_proofs")
                    .long("preserve-intermediary-proofs")
                    .action(clap::ArgAction::SetTrue),
            );

        Self { command }
    }

    pub fn with_proof_storage_options(self) -> Self {
        let command = self
            .command
            .arg(
                Arg::with_name("proof_storage_type")
                    .long("proof-storage-type")
                    .value_name("proof_storage_type")
                    .help("Sets the type of proof storage")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["redis", "file", "azure", "aws"]),
            )
            .arg(
                Arg::with_name("folder_name")
                    .long("folder-name")
                    .value_name("folder_name")
                    .help("Sets the name of the folder proofs will be stored in")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("azure_account")
                    .long("azure-account-name")
                    .value_name("azure_account")
                    .help("Sets the name of the azure account")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("azure_container")
                    .long("azure-container-name")
                    .value_name("azure_container")
                    .help("Sets the name of the azure container")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("aws_endpoint_url")
                    .long("aws-endpoint-url")
                    .value_name("aws_endpoint_url")
                    .help("Sets the aws endpoint url")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("aws_region")
                    .long("aws-region")
                    .value_name("aws_region")
                    .help("Sets the aws region")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("aws_bucket_name")
                    .long("aws-bucket-name")
                    .value_name("aws_bucket_name")
                    .help("Sets the aws bucket name")
                    .takes_value(true),
            );

        Self { command }
    }

    pub fn with_redis_options(self, config: &HashMap<String, String>) -> Self {
        let command = self.command.arg(
            Arg::with_name("redis_connection")
                .short('r')
                .long("redis")
                .value_name("Redis Connection")
                .help("Sets a custom Redis connection")
                .takes_value(true)
                .default_value(Box::leak(Box::new(format!(
                    "redis://{}:{}/",
                    config["redis-host"], config["redis-port"]
                )))),
        );

        Self { command }
    }

    pub fn with_work_queue_options(self) -> Self {
        let command = self.command.arg(
            Arg::with_name("stop_after")
                .long("stop-after")
                .value_name("Stop after")
                .help("Sets how much seconds to wait until the program stops if no new tasks are found in the queue")
                .takes_value(true)
                .default_value("20")
        )
            .arg(
                Arg::with_name("lease_for")
                    .value_name("lease-for")
                    .help("Sets for how long the task will be leased and then possibly requeued if not finished")
                    .takes_value(true)
                    .default_value("30"));

        Self { command }
    }

    pub fn get_matches(self) -> ArgMatches {
        self.command.get_matches()
    }
}

pub fn parse_config_file(filepath: String) -> Result<HashMap<String, String>> {
    let mut content = String::new();
    let mut file = File::open(filepath)?;
    file.read_to_string(&mut content)?;
    Ok(serde_json::from_str(&content.as_str())?)
}

pub fn gindex_from_validator_index(index: u64, depth: u32) -> u64 {
    return 2u64.pow(depth) - 1 + index;
}

pub fn format_hex(str: String) -> String {
    if str.starts_with("0x") {
        return str[2..].to_string();
    }

    return str;
}
