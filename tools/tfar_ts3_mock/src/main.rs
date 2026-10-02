//! The `tfar_ts3_mock` binary: loads one plugin into the fake TeamSpeak and prints what it did.
//!
//! **Role:** a smoke run for a built plugin: `tfar_ts3_mock <plugin.so>` prints the plugin's
//! metadata, starts it (`setFunctionPointers`, `registerPluginID`, `init`), connects a fake
//! server, shuts it down, and prints the call log and the leak report.
//!
//! **Position:** the binary of `tools/tfar_ts3_mock`, over its library; developers run it by
//! hand on a plugin from `cargo xtask dist` or a debug build.
//!
//! **Signals and state:** the process's one mock host; the plugin's own threads run between
//! init and shutdown.
//!
//! **Invariants:** exit code `0` when the plugin starts, connects and shuts down with a clean leak
//! report, `1` when any step fails or memory leaks, `2` for a wrong command line.

use std::path::PathBuf;
use std::process::ExitCode;

use tfar_ts3_mock::{MockHost, PluginLibrary, PluginMetadata, ServerSetup};

/// The usage line printed for a wrong command line.
const USAGE: &str = "usage: tfar_ts3_mock <plugin.so>";

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(plugin_path), None) = (arguments.next(), arguments.next()) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    if plugin_path == "--help" || plugin_path == "-h" {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match run(&PathBuf::from(plugin_path)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("tfar_ts3_mock: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the smoke scenario on the plugin at `plugin_path`; `Ok(false)` when memory leaked.
fn run(plugin_path: &std::path::Path) -> tfar_ts3_mock::Result<bool> {
    let plugin = PluginLibrary::open(plugin_path)?;
    print_metadata(plugin.metadata());
    let mut host = MockHost::start(plugin, &tfar_ts3_mock::HostOptions::default())?;
    println!("init: ok");
    let server = host.connect_server(&ServerSetup::default())?;
    println!("connected: server {}", server.0);
    host.shutdown();
    println!("shutdown: ok");
    println!("calls:");
    for call in host.calls() {
        println!("  {call:?}");
    }
    let leaks = host.leak_report();
    if leaks.is_clean() {
        println!("leaks: none");
    } else {
        println!("leaks: {leaks:?}");
    }
    Ok(leaks.is_clean())
}

/// Prints what the plugin says about itself.
fn print_metadata(metadata: &PluginMetadata) {
    println!("name: {}", metadata.name);
    println!("version: {}", metadata.version);
    println!("apiVersion: {}", metadata.api_version);
    println!("author: {}", metadata.author);
    println!("description: {}", metadata.description);
    println!("autoload: {}", metadata.requests_autoload);
}
