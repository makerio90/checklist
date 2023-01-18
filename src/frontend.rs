use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

use crate::Checklist;

/// checklist thats currently active
struct ActiveChecklist {
    name: String,
    reset_on: Option<DateTime<Utc>>,
    todo: HashMap<String, bool>,
}

pub struct Frontend {
    checklists: Vec<ActiveChecklist>,
}

impl Frontend {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, checklists: Vec<Checklist>) -> Self {
        let checklists = checklists
            .iter()
            .map(|checklist| {
                let mut map: HashMap<String, bool> = HashMap::new();
                for todo in checklist.todo.iter() {
                    map.insert(todo.clone(), false);
                }
                let reset_on = checklist.reset_schedule.clone().and_then(|schedule| {
                    Schedule::from_str(&format!("* {} * ", schedule))
                        .unwrap()
                        .upcoming(Utc)
                        .next()
                });
                ActiveChecklist {
                    name: checklist.name.clone(),
                    reset_on,
                    todo: map,
                }
            })
            .collect();
        Self { checklists }
    }
}

impl eframe::App for Frontend {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        let now = Utc::now();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("checklists");
            for checklist in &mut self.checklists {
                ui.group(|ui| {
                    ui.heading(&checklist.name);
                    if let Some(date) = checklist.reset_on {
                        let duration = Duration::seconds(date.timestamp() - now.timestamp());
                        let seconds = duration.num_seconds() % 60;
                        let minutes = (duration.num_seconds() / 60) % 60;
                        let hours = (duration.num_seconds() / 60) / 60;
                        ui.label(format!("resets in {}:{}:{}", hours, minutes, seconds));
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for mut todo in &mut checklist.todo {
                            ui.checkbox(&mut todo.1, todo.0);
                        }
                    });
                });
            }
        });
    }
}
