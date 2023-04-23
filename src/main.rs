// reggie - fast Windows registry searcher
// Copyright (C) 2023  Donny Johnson
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::error::Error;
use std::fmt::Display;
use std::time::Instant;
use std::io::Write;

use clap::Parser;
use rayon::ThreadPoolBuilder;
use winreg::HKEY;
use winreg::enums::*;
use regex::Regex;

pub mod winreg_provider;
pub mod winreg_provider2;

#[derive(Parser)]
struct Config {
    #[arg(short='p', long)]
    print: bool,
    #[arg(short='c', long)]
    print_count: bool,
    #[arg(short='t', long)]
    print_time: bool,
    #[arg(short='f', long)]
    filter: Option<String>,
    #[arg(short='B', long)]
    backend: Option<String>,
    #[arg(short='T', long)]
    num_threads: Option<usize>,
    #[arg(short='H', long)]
    hive: Option<String>,
    #[arg(short='l', long)]
    list_hives: bool,
    #[arg(short='V', long)]
    version: bool,
}

#[derive(Debug)]
enum CliError {
    InvalidBackend,
    UnknownHiveName,
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for CliError {}

fn create_regex_filter(args: &mut Config) -> Regex {
    let filter = args.filter.as_ref()
        .map_or_else(
            || {
                Regex::new("")
            },
            |s| {
                args.print = true;
                Regex::new(&format!("(?i){}", s))
            })
        .map_err(
            |error| {
                eprintln!("error with filter: {}", error);
                error
            })
        .unwrap();
    filter
}

struct RegistryHiveInfo {
    short_name: &'static str,
    name: &'static str,
    hkey: HKEY,
}

impl RegistryHiveInfo {
    fn new(short_name: &'static str, name: &'static str, hkey: HKEY) -> RegistryHiveInfo {
        RegistryHiveInfo {
            short_name,
            name,
            hkey,
        }
    }
}

fn get_hives() -> Vec<RegistryHiveInfo> {
    vec![
        RegistryHiveInfo::new("HKLM", "HKEY_LOCAL_MACHINE", HKEY_LOCAL_MACHINE),
        RegistryHiveInfo::new("HKCR", "HKEY_CLASSES_ROOT", HKEY_CLASSES_ROOT),
        RegistryHiveInfo::new("HKCC", "HKEY_CURRENT_CONFIG", HKEY_CURRENT_CONFIG),
        RegistryHiveInfo::new("HKCU", "HKEY_CURRENT_USER", HKEY_CURRENT_USER),
        RegistryHiveInfo::new("HKCULL", "HKEY_CURRENT_USER_LOCAL_SETTINGS", HKEY_CURRENT_USER_LOCAL_SETTINGS),
        RegistryHiveInfo::new("HKDD", "HKEY_DYN_DATA", HKEY_DYN_DATA),
        RegistryHiveInfo::new("HKPD", "HKEY_PERFORMANCE_DATA", HKEY_PERFORMANCE_DATA),
        RegistryHiveInfo::new("HKPL", "HKEY_PERFORMANCE_NLSTEXT", HKEY_PERFORMANCE_NLSTEXT),
        RegistryHiveInfo::new("HKPT", "HKEY_PERFORMANCE_TEXT", HKEY_PERFORMANCE_TEXT),
        RegistryHiveInfo::new("HKU", "HKEY_USERS", HKEY_USERS),
    ]
}

fn get_hive(name: &str) -> Result<HKEY,CliError> {
    let hive = get_hives()
        .into_iter()
        .filter(|x| x.short_name == name || x.name == name)
        .next();
    if let Some(hive) = hive {
        Ok(hive.hkey)
    }
    else {
        Err(CliError::UnknownHiveName)
    }
}

fn print_hives(prefix: &str, stream: &mut dyn Write) -> Result<(),Box<dyn Error>> {
    let hives = get_hives();
    for hive in hives {
        writeln!(stream, "{}{}, {}, {}", prefix, hive.short_name, hive.name, hive.hkey)?;
    }
    Ok(())
}

fn print_version() {
    println!("reggie-0.1.0  Copyright (C) 2023  Donny Johnson");
    println!("This program comes with ABSOLUTELY NO WARRANTY.");
    println!("This is free software, and you are welcome to redistribute it under certain");
    println!("conditions.  See LICENSE.txt or <https://www.gnu.org/licenses/gpl-3.0.en.html>.");
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Config::parse();

    if args.version {
        print_version();
        return Ok(());
    }
    else if args.list_hives {
        print_hives("", &mut std::io::stdout())?;
        return Ok(());
    }

    let hive_name = args.hive.clone().unwrap_or("HKLM".to_string());

    let filter = create_regex_filter(&mut args);
    let timer = args.print_time.then_some(Instant::now());
    let thread_count = args.num_threads.unwrap_or(0);
    let backend = args.backend.unwrap_or("v2".to_string());
    let hive = get_hive(&hive_name);

    let hive = match hive {
        Ok(hive) => hive,
        Err(error) => {
            eprintln!("unknown hive: {}", hive_name);
            eprintln!("Valid hives:");
            print_hives("    ", &mut std::io::stderr())?;
            return Err(error.into());
        },
    };

    let mut results = if backend == "v1" {
        winreg_provider::get_all(hive)
    }
    else if backend == "v2" {
        if thread_count != 0 {
            ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build_global().expect("issue with thread count argument");
        }
        winreg_provider2::get_all(hive)
    }
    else {
        eprintln!("unknown backend: {}", backend);
        return Err(CliError::InvalidBackend.into());
    };

    let time = timer.map(|x| x.elapsed().as_secs_f64());

    let count = results.len();

    if args.print {
        results.retain(|x| filter.is_match(x));

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        for result in results {
            writeln!(handle, "{}", result)?;
        }
    }

    if args.print_count {
        eprintln!("There are {} keys in {}.", count, hive_name)
    }

    if let Some(seconds) = time {
        eprintln!("Took {} seconds", seconds);
        let kps = (count as f64) / seconds;
        eprintln!("{} keys/second", (kps as i32));
    }

    Ok(())
}
