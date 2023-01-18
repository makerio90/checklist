use config::{Config, File};
use log::{error, info, log_enabled, warn, Level};
use serde::Deserialize;
use std::{env, path::Path};

#[derive(Deserialize)]
struct Checklist {
    /// name of the todo
    name: String,
    /// if the user wants to, they can have the todo list restart on a schedule
    reset_schedule: Option<String>,
    /// the actule checklist
    todo: Vec<String>,
}

#[derive(Deserialize)]
struct Conf {
    checklist: Vec<Checklist>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // if env var `RUST_LOG` is not set, set it
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    // Start the logger
    env_logger::init();
    // log some info
    info!(target: "init",
        "checklist v{}",
        env!("CARGO_PKG_VERSION")
    );

    // path to the config file
    // TODO: dont hardcode this
    let dir = Path::new(&env::var("HOME")?).join(".config/checklist/todo.toml");

    // if theres no config file, warn the user there is none
    // TODO: if there is none, make a default one
    if !dir.exists() {
        error!("no config file! checked {}", dir.to_str().unwrap());
        panic!()
    }
    // load the config
    let s = Config::builder()
        .add_source(File::with_name(dir.to_str().unwrap()))
        .build()?;
    // deserialise the config
    let mut checklists: Conf = s.try_deserialize()?;

    info!("loaded config!");

    /*
     * OwO whats this? :3333
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
    );
    */

    Ok(())
}
