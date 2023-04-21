use std::error::Error;

use clap::Parser;
use winreg::enums::*;
use winreg::RegKey;
use std::time::Instant;
use regex::Regex;

#[derive(Parser)]
struct Config {
    #[arg(short='p', long)]
    print_each: bool,
    #[arg(short='c', long)]
    print_count: bool,
    #[arg(short='t', long)]
    print_time: bool,
    #[arg(short='f', long)]
    filter: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Config::parse();
    let mut timer = None;
    let filter;
    if args.filter.is_some() {
        // Set to ignore case.
        filter = Regex::new(&format!("(?i){}", args.filter.as_ref().unwrap()));
    }
    else {
        filter = Regex::new(".*");
    }

    if filter.is_err() {
        let error = filter.unwrap_err();
        eprintln!("Filter has issue: {}", error);
        return Err(Box::new(error));
    }

    let filter = filter.unwrap();

    if args.print_time {
        timer = Some(Instant::now());
    }

    let results = get_all(HKEY_LOCAL_MACHINE);

    if args.print_each || args.filter.is_some() {
        results
            .iter()
            .filter(|x| filter.is_match(x))
            .for_each(|x| println!("{}", x));
    }

    if args.print_count {
        println!("There are {} keys in HKLM.", results.len());
    }

    timer.map(|t| println!("Took {} seconds", t.elapsed().as_secs_f64()));
    Ok(())
}

fn get_all(hive: isize) -> Vec<String> {
    let root = RegKey::predef(hive);
    let root_name;
    match hive {
        HKEY_CLASSES_ROOT => root_name = String::from("HKCR"),
        HKEY_CURRENT_CONFIG => root_name = String::from("HKCC"),
        HKEY_CURRENT_USER => root_name = String::from("HKCU"),
        HKEY_CURRENT_USER_LOCAL_SETTINGS => root_name = String::from("HKCULL"),
        HKEY_DYN_DATA => root_name = String::from("HKDD"),
        HKEY_LOCAL_MACHINE => root_name = String::from("HKLM"),
        HKEY_PERFORMANCE_DATA => root_name = String::from("HKPD"),
        HKEY_PERFORMANCE_NLSTEXT => root_name = String::from("HKPL"),
        HKEY_PERFORMANCE_TEXT => root_name = String::from("HKPT"),
        HKEY_USERS => root_name = String::from("HKU"),
        _ => root_name = String::from("(unknown)"),
    }
    let mut output = Vec::new();
    get_all_with_prefix(root, root_name, &mut output);
    output
}

fn get_all_with_prefix(root: RegKey, prefix: String, vector: &mut Vec<String>) {
    vector.push(prefix.clone());
    let keys = root.enum_keys();
    let keys: Vec<String> = keys.filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
    for key in &keys {
        let subkey = root.open_subkey(key);
        if subkey.is_err() {
            continue;
        }
        let subkey = subkey.unwrap();
        let path = format!("{}\\{}", prefix, key);
        get_all_with_prefix(subkey, path, vector);
    }
}
