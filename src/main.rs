mod frontend;
use chrono::{DateTime, Utc};
use config::{Config, File as ConfigFile};
use cron::{OwnedScheduleIterator, Schedule};
use log::{error, info, log_enabled, warn, Level};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap},
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
    str::FromStr,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

#[derive(Deserialize)]
pub struct ConfigChecklist {
    /// name of the todo
    name: String,
    /// if the user wants to, they can have the todo list restart on a schedule
    reset_schedule: Option<String>,
    /// the actule checklist
    todo: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Checklist {
    name: String,
    reset_on: Option<DateTime<Utc>>,
    #[serde(skip)]
    reset_every: Option<OwnedScheduleIterator<Utc>>,
    tasks: HashMap<String, bool>,
}

#[derive(Deserialize)]
struct Conf {
    checklist: Vec<ConfigChecklist>,
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
    let config_dir = Path::new(&env::var("HOME")?).join(".config/checklist/");

    // if theres no config file, warn the user there is none
    // TODO: if there is none, make a default one
    if !config_dir.exists() {
        error!("no config dir checked {}", config_dir.to_str().unwrap());
        panic!()
    }
    // load the config
    let s = Config::builder()
        .add_source(ConfigFile::with_name(
            config_dir.join("config.toml").to_str().unwrap(),
        ))
        .build()?;
    // deserialise the config
    let mut checklist_conf: Conf = s.try_deserialize()?;

    info!("loaded config!");

    let checklists: Result<Vec<Checklist>, serde_json::Error> = checklist_conf
        .checklist
        .iter()
        .map(|checklist| {
            let checklist_dir = config_dir.join(checklist.name.clone() + ".json");
            if checklist_dir.exists() {
                // if a checklist file already exists, just load it
                // TODO: if the file is older than the config file, dont use it.
                let file =
                    File::open(checklist_dir.to_str().unwrap()).expect("could not open file");
                let buf = BufReader::new(file);
                serde_json::from_reader(buf)
            } else {
                // if not, create a new one
                let mut tasks: HashMap<String, bool> = HashMap::new();
                for task in &checklist.todo {
                    tasks.insert(task.to_string(), false);
                }
                let reset_every = checklist.reset_schedule.clone().map(|schedule| {
                    Schedule::from_str(&format!("* {} * ", schedule))
                        .unwrap()
                        .upcoming_owned(Utc)
                });
                Ok(Checklist {
                    tasks,
                    reset_on: None,
                    reset_every,
                    name: checklist.name.clone(),
                })
            }
        })
        .collect();

    let checklists = Arc::new(RwLock::new(checklists?));

    {
        let checklists = checklists.clone();
        thread::spawn(move || handle_checklists(checklists));
    }

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "checklist",
        native_options,
        Box::new(|cc| Box::new(frontend::Frontend::new(cc, checklists))),
    );

    Ok(())
}

fn handle_checklists(checklists: Arc<RwLock<Vec<Checklist>>>) {
    // update everything every 10 secconds
    // TODO: dynamic file saveing
    let speed = Duration::from_secs(10);
    let config_dir = Path::new(&env::var("HOME").unwrap()).join(".config/checklist/");
    loop {
        for checklist in &mut *checklists.write().unwrap() {
            if checklist.reset_on == None && checklist.reset_every.is_some() {
                checklist.reset_on = Some(checklist.reset_every.as_mut().unwrap().next().unwrap())
            }

            let checklist_dir = config_dir.join(checklist.name.clone() + ".json");
            let json = serde_json::to_string(&checklist);

            fs::write(checklist_dir.to_str().unwrap(), json.unwrap()).expect("cant write to file!")
        }
        thread::sleep(speed)
    }
}
