extern crate rutie;

use lazy_static::lazy_static;
use log::{error, log};
use rutie::{AnyException, AnyObject, Object, RString, VM};
use std::fs::File;
use std::io::{BufReader, Read};
use std::{fs, io};

const CACHE_PLUGINS: bool = false;

lazy_static! {
    static ref PLUGIN_IDENTIFIERS: Vec<String> = init();
}

fn init() -> Vec<String> {
    VM::init();

    let plugin_paths = load_plugin_paths();

    let plugin_codes = read_plugins(plugin_paths.clone());

    feed_plugins(plugin_codes);

    let identifiers = get_plugin_identifiers(plugin_paths);

    init_plugins(&identifiers);

    identifiers
}

pub fn pasta_filter(s: &str) -> bool {
    true
}

pub fn on_pasta_read(s: &str) -> String {
    let mut processed_content: String = String::from(s);

    for PLUGIN_IDENTIFIER in PLUGIN_IDENTIFIERS.iter() {
        processed_content = eval_for_string(PLUGIN_IDENTIFIER, "on_pasta_read", s);
    }

    processed_content
}

pub fn on_pasta_created(s: &str) -> String {
    let mut processed_content: String = String::from(s);

    for PLUGIN_IDENTIFIER in PLUGIN_IDENTIFIERS.iter() {
        processed_content = eval_for_string(PLUGIN_IDENTIFIER, "on_pasta_created", s);
    }

    processed_content
}

pub fn init_plugins(plugin_identifiers: &Vec<String>) {
    for PLUGIN_IDENTIFIER in plugin_identifiers.iter() {
        eval_for_string(PLUGIN_IDENTIFIER, "init", "");

        let init_result = eval_for_string(&PLUGIN_IDENTIFIER, "init", "");
        let id = eval_for_string(&PLUGIN_IDENTIFIER, "get_id", "");
        let name = eval_for_string(&id, "get_name", "");
        let version = eval_for_string(&id, "get_version", "");

        log::info!("Initialised plugin {id} - {name} ({version})");
    }
}

fn eval_for_string(plugin_id: &str, function: &str, parameter: &str) -> String {
    match VM::eval(&*format!("MBP::{}::{}({})", plugin_id, function, parameter)) {
        Ok(result) => match result.try_convert_to::<RString>() {
            Ok(ruby_string) => ruby_string.to_string(),
            Err(err) => err.to_string(),
        },
        Err(err) => {
            log::error!(
                "Failed to run function '{}' on plugin {}: {}",
                function,
                plugin_id,
                err
            );
            err.to_string()
        }
    }
}

fn load_plugin_paths() -> Vec<String> {
    let paths = fs::read_dir("./plugins").expect("Failed to access ./plugins library.");

    let mut plugin_paths: Vec<String> = Vec::new();

    for path in paths {
        plugin_paths.push(path.unwrap().path().to_str().unwrap().parse().unwrap());
    }

    plugin_paths
}

fn read_plugins(plugin_paths: Vec<String>) -> Vec<String> {
    let mut plugin_codes: Vec<String> = Vec::new();

    for plugin_path in plugin_paths {
        let plugin_code = match fs::read_to_string(&plugin_path) {
            Ok(result) => result,
            Err(err) => {
                log::error!("Failed to read plugin file {}: {}", plugin_path, err);
                continue;
            }
        };
        plugin_codes.push(plugin_code);
    }

    plugin_codes
}

fn feed_plugins(plugin_codes: Vec<String>) {
    for plugin_code in plugin_codes {
        match VM::eval(plugin_code.as_str()) {
            Ok(result) => {}
            Err(error) => {
                log::error!("Failed to initialise plugin: {}", error);
                continue;
            }
        }
    }
}

fn get_plugin_identifiers(plugin_paths: Vec<String>) -> Vec<String> {
    let mut plugin_ids: Vec<String> = Vec::new();
    for plugin_path in plugin_paths {
        plugin_ids.push(plugin_path.replace("./plugins/", "").replace(".rb", ""))
    }
    plugin_ids
}
