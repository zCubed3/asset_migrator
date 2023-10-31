use std::path::PathBuf;
use eframe::Frame;

use eframe::egui;

use rfd;

//
// AMGuiApp Structure
//

/// The app implementation for Asset Migrator GUI
#[derive(Default)]
pub struct AMGuiApp {
    src_project: Option<PathBuf>,
    dst_project: Option<PathBuf>
}

impl eframe::App for AMGuiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //
            // Menu bar
            //
            egui::menu::bar(ui, |ui| {
                //
                // Build info sub-menu
                //
                ui.menu_button("Build Info", |ui| {
                    ui.label(format!("Branch '{}'", env!("GIT_BRANCH")));
                    ui.add_space(10.0);

                    ui.label(format!("Commit '{}'", env!("GIT_COMMIT")));
                    ui.add_space(10.0);

                    ui.label(format!("Dirty '{}'", env!("GIT_DIRTY")));
                    ui.add_space(10.0);

                    ui.label(format!("Timestamp '{}'", env!("SOURCE_TIMESTAMP")));
                });
            });

            //
            // Asset Migrator Panel
            //
            let heading_str = {
                // TODO: Make this less verbose in release builds
                format!("Asset Migrator GUI - WIP < {} : {} >", env!("GIT_COMMIT"), env!("SOURCE_TIMESTAMP"))
            };

            ui.heading(heading_str);

            if ui.button("Open Source Project").clicked() {
                self.src_project = rfd::FileDialog::new().pick_folder();
            }

            if ui.button("Open Destination Project").clicked() {
                self.dst_project = rfd::FileDialog::new().pick_folder();
            }

            ui.label(format!("Src Project = {:?}", self.src_project));
            ui.label(format!("Dst Project = {:?}", self.dst_project));
        });
    }
}