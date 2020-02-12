/*
 * Copyright 2018-2019 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.  You may obtain a copy of the
 * License at: https://ton.dev/licenses
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */
#[macro_use]
extern crate clap;

mod account;
mod config;
mod deploy;
mod genaddr;
mod helpers;
mod call;

use account::get_account;
use call::call_contract;
use clap::ArgMatches;
use config::{Config, set_config};
use deploy::deploy_contract;
use genaddr::generate_address;

fn main() -> Result<(), i32> {    
    main_internal().map_err(|err_str| {
        println!("Error: {}", err_str);
        1
    })
}

fn main_internal() -> Result <(), String> {
    let build_info = match option_env!("BUILD_INFO") {
        Some(s) => s,
        None => "none",
    };

    let matches = clap_app! (tonlabs_cli =>        
        (version: &*format!("0.1 ({})", build_info))
        (author: "TONLabs")
        (about: "TONLabs console tool for TON")
        (@subcommand version =>
            (about: "Prints build and version info.")
        )
        (@subcommand genaddr =>
            (@setting AllowNegativeNumbers)
            (about: "Calculates smart contract address in different formats.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Compiled smart contract (tvc file).")
            (@arg WC: --wc +takes_value "Workchain id used to generate user-friendly addresses (default -1).")
            (@arg GENKEY: --genkey +takes_value conflicts_with[SETKEY] "Generates new keypair for the contract and saves it to the file.")
            (@arg SETKEY: --setkey +takes_value conflicts_with[GENKEY] "Loads existing keypair from the file.")
        )
        (@subcommand deploy =>
            (@setting AllowNegativeNumbers)
            (about: "Deploy smart contract to blockchain.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Compiled smart contract (tvc file)")
            (@arg PARAMS: +required +takes_value "Constructor arguments.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign 'constructor message'.")            
        )
        (@subcommand call =>
            (about: "Sends external message to contract with encoded function call.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign message.")
        )
        (@subcommand run =>
            (about: "Runs contract's get-method.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value conflicts_with[BODY] "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value conflicts_with[BODY] "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value conflicts_with[BODY] "Json file with contract ABI.")
        )
        (@subcommand config =>
            (about: "Writes parameters to config file that can be used later in subcommands.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg URL: --url +takes_value "Url to connect.")
            (@arg ABI: --abi +takes_value conflicts_with[DATA] "File with contract ABI.")
            (@arg KEYS: --keys +takes_value "File with keypair.")
            (@arg ADDR: --addr +takes_value "Contract address.")
        )
        (@subcommand account =>
            (about: "Gets account information.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Smart contract address.")
        )
        (@setting SubcommandRequired)
    ).get_matches();

    let conf = Config::from_file("tonlabs-cli.conf.json").unwrap_or(Config::new());

    if let Some(m) = matches.subcommand_matches("call") {
        return send_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("run") {
        return run_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("deploy") {        
        return deploy_command(m, conf);
    } 
    if let Some(m) = matches.subcommand_matches("config") {
        return config_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("genaddr") {
        return genaddr_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("account") {
        return account_command(m, conf);
    }
    if let Some(_) = matches.subcommand_matches("version") {
        println!(
            "tonlabs-cli {}\nCOMMIT_ID: {}\nBUILD_DATE: {}\nCOMMIT_DATE: {}\nGIT_BRANCH: {}",
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_GIT_COMMIT"),
            env!("BUILD_TIME") ,
            env!("BUILD_GIT_DATE"),
            env!("BUILD_GIT_BRANCH")
        );
        return Ok(());
    }
    Err("invalid arguments".to_string())
}

fn send_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let addr = matches.value_of("ADDRESS").unwrap();
    let method = matches.value_of("METHOD").unwrap();
    let params = matches.value_of("PARAMS").unwrap();
    let abi = matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?;
    let keys = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());
    call_contract(config, addr, &abi, method, params, keys, false)
}

fn run_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let addr = matches.value_of("ADDRESS").unwrap();
    let method = matches.value_of("METHOD").unwrap();
    let params = matches.value_of("PARAMS").unwrap();
    let abi = matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?;
    call_contract(config, addr, &abi, method, params, None, true)
}

fn deploy_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC").unwrap();
    let params = matches.value_of("PARAMS").unwrap();
    let abi = matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?;
    let keys = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone())
        .ok_or("keypair file not defined. Supply it in config file or command line.".to_string())?;
    deploy_contract(config, tvc, &abi, params, &keys)
}

fn config_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let url = matches.value_of("URL");
    let addr = matches.value_of("ADDR");
    let keys = matches.value_of("KEYS");
    let abi = matches.value_of("ABI");
    set_config(config, "tonlabs-cli.conf.json", url, addr, abi, keys)
}

fn genaddr_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC").unwrap();
    let wc = matches.value_of("WC");
    let keys = matches.value_of("GENKEY").or(matches.value_of("SETKEY"));
    let new_keys = matches.is_present("GENKEY");
    generate_address(config, tvc, wc, keys, new_keys)
}

fn account_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let addr = matches.value_of("ADDRESS").unwrap();
    get_account(config, addr)
}