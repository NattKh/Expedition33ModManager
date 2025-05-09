use clap::{Parser, Subcommand};
mod core;

use eframe::egui;
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

const CACHE_FILE: &str = "unnie_mod_manager_cache.json";

#[derive(Parser)]
#[command(name = "UnnieModManager")]
#[command(about = "A CLI tool to manage UE4SS and mods for Expedition 33", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install or update UE4SS in the target game directory
    InstallUe4ss {
        /// Path to the game Win64 directory
        #[arg(short, long)]
        target_dir: String,
    },
    /// Install a mod from a zip file (future: drag-and-drop in GUI)
    InstallMod {
        /// Path to the mod zip file
        #[arg(short, long)]
        zip_path: String,
        /// Path to the game Win64 directory
        #[arg(short, long)]
        target_dir: String,
    },
    /// List installed mods in the Mods folder
    ListMods {
        /// Path to the game Win64 directory
        #[arg(short, long)]
        target_dir: String,
    },
    /// Launch the GUI
    Gui,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppCache {
    pub last_selected_game: Option<usize>,
    pub last_win64_dir: String,
    pub last_installed_mods: Vec<String>,
    pub last_scanned_files: Vec<String>,
    pub last_debug_output: String,
}

fn load_cache() -> AppCache {
    if Path::new(CACHE_FILE).exists() {
        let data = fs::read_to_string(CACHE_FILE).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppCache::default()
    }
}

fn save_cache(cache: &AppCache) {
    if let Ok(data) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(CACHE_FILE, data);
    }
}

#[cfg(windows)]
fn is_elevated() -> bool {
    extern "system" {
        fn IsUserAnAdmin() -> i32;
    }
    unsafe { IsUserAnAdmin() != 0 }
}

#[cfg(not(windows))]
fn is_elevated() -> bool {
    true
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::InstallUe4ss { target_dir } => {
            core::install_ue4ss(&target_dir).expect("Failed to install UE4SS");
        }
        Commands::InstallMod { zip_path, target_dir } => {
            match core::install_mod_from_zip(&zip_path, &target_dir) {
                Ok(_) => println!("Mod installed successfully."),
                Err(e) => println!("[ERROR] Failed to install mod: {}", e),
            }
        }
        Commands::ListMods { target_dir } => {
            match core::list_installed_mods(&target_dir) {
                Ok(mods) => {
                    if mods.is_empty() {
                        println!("No mods installed.");
                    } else {
                        println!("Installed mods:");
                        for m in mods {
                            println!("- {}", m);
                        }
                    }
                }
                Err(e) => println!("Error listing mods: {}", e),
            }
        }
        Commands::Gui => {
            run_gui();
        }
    }
}

fn run_gui() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Expedition 33 UnnieModManager",
        native_options,
        Box::new(|_cc| Box::new(GuiApp::default())),
    ).unwrap();
}

struct GuiApp {
    win64_dir: String,
    debug_output: String,
    installed_mods: Vec<String>,
    scanned_files: Vec<String>,
    cache: AppCache,
    debug_mode: bool,
    ui_scale: f32,
}

impl Default for GuiApp {
    fn default() -> Self {
        let cache = load_cache();
        Self {
            win64_dir: cache.last_win64_dir.clone(),
            debug_output: cache.last_debug_output.clone(),
            installed_mods: cache.last_installed_mods.clone(),
            scanned_files: cache.last_scanned_files.clone(),
            cache,
            debug_mode: false,
            ui_scale: 1.0,
        }
    }
}

// Helper macro for debug printing
macro_rules! debug_println {
    ($app:expr, $($arg:tt)*) => {
        if $app.debug_mode {
            $app.debug_output.push_str(&format!($($arg)*));
        }
    };
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set a custom dark theme for better contrast
        ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::proportional(28.0)),
            (egui::TextStyle::Body, egui::FontId::proportional(20.0)),
            (egui::TextStyle::Button, egui::FontId::proportional(20.0)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(18.0)),
            (egui::TextStyle::Small, egui::FontId::proportional(16.0)),
        ].into();
        ctx.set_style(style);

        // Accent color for buttons
        let accent_color = egui::Color32::from_rgb(0, 120, 220); // blue accent

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("Expedition 33 UnnieModManager");
            ui.add_space(8.0);
        });

        egui::SidePanel::left("left_panel").min_width(280.0).show(ctx, |ui| {
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.heading("Settings");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("UI Scale:");
                    let mut scale_changed = false;
                    scale_changed |= ui.add(egui::Slider::new(&mut self.ui_scale, 0.8..=2.0).text("")).changed();
                    if scale_changed {
                        ctx.set_pixels_per_point(self.ui_scale);
                    }
                });
                ui.checkbox(&mut self.debug_mode, "Debug Mode");
            });
            ui.add_space(16.0);
            ui.group(|ui| {
                ui.heading("Game Directory");
                ui.add_space(8.0);
                ui.label("Game Win64 Directory:");
                let changed = ui.text_edit_singleline(&mut self.win64_dir).changed();
                ui.add_space(4.0);
                if ui.add_sized([220.0, 32.0], egui::Button::new("Select Win64 Directory")).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.win64_dir = dir.display().to_string();
                        debug_println!(self, "[INFO] Selected directory: {}\n", self.win64_dir);
                        self.cache.last_win64_dir = self.win64_dir.clone();
                        self.cache.last_debug_output = self.debug_output.clone();
                        save_cache(&self.cache);
                    }
                }
                if changed {
                    self.update_mod_list();
                    self.cache.last_win64_dir = self.win64_dir.clone();
                    self.cache.last_debug_output = self.debug_output.clone();
                    save_cache(&self.cache);
                }
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Example game path: Expedition 33\\Sandfall\\Binaries\\Win64").color(egui::Color32::GRAY).italics());
            });
            ui.add_space(16.0);
            ui.group(|ui| {
                ui.heading("Mod Management");
                ui.add_space(8.0);
                let button_frame = |ui: &mut egui::Ui, text: &str| {
                    ui.add_sized([
                        220.0, 36.0
                    ], egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE)).fill(accent_color))
                };
                if button_frame(ui, "Install UE4SS").clicked() {
                    self.debug_output.clear();
                    if self.win64_dir.is_empty() {
                        self.debug_output.push_str("[ERROR] Please select a Win64 directory first.\n");
                    } else {
                        debug_println!(self, "[INFO] Installing UE4SS...\n");
                        match core::install_ue4ss(&self.win64_dir) {
                            Ok(_) => {
                                self.debug_output.push_str("[INFO] UE4SS installed successfully.\n");
                                self.update_mod_list();
                                let entries = core::list_all_files_and_dirs(&self.win64_dir).unwrap_or_default();
                                self.scanned_files = entries;
                            },
                            Err(e) => self.debug_output.push_str(&format!("[ERROR] Failed to install UE4SS: {}\n", e)),
                        }
                        self.cache.last_win64_dir = self.win64_dir.clone();
                        self.cache.last_scanned_files = self.scanned_files.clone();
                        self.cache.last_debug_output = self.debug_output.clone();
                        save_cache(&self.cache);
                    }
                }
                ui.add_space(8.0);
                if button_frame(ui, "Install Mod").clicked() {
                    self.debug_output.clear();
                    if self.win64_dir.is_empty() {
                        self.debug_output.push_str("[ERROR] Please select a Win64 directory first.\n");
                    } else if let Some(zip_path) = rfd::FileDialog::new().add_filter("Zip files", &["zip"]).pick_file() {
                        let path_str = zip_path.display().to_string();
                        let file_name = zip_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        debug_println!(self, "[INFO] Selected mod zip: {}\n", path_str);
                        match core::install_mod_from_zip(&path_str, &self.win64_dir) {
                            Ok(_) => self.debug_output.push_str(&format!("[INFO] Mod '{}' installed successfully.\n", file_name)),
                            Err(e) => self.debug_output.push_str(&format!("[ERROR] Failed to install mod '{}': {}\n", file_name, e)),
                        }
                        self.update_mod_list();
                        self.cache.last_win64_dir = self.win64_dir.clone();
                        self.cache.last_installed_mods = self.installed_mods.clone();
                        self.cache.last_debug_output = self.debug_output.clone();
                        save_cache(&self.cache);
                    }
                }
                ui.add_space(8.0);
                if button_frame(ui, "Open Mods Folder").clicked() {
                    if self.win64_dir.is_empty() {
                        self.debug_output.clear();
                        self.debug_output.push_str("[ERROR] Please select a Win64 directory first.\n");
                    } else {
                        let mods_path = std::path::Path::new(&self.win64_dir).join("Mods");
                        if !mods_path.exists() {
                            if let Err(e) = std::fs::create_dir_all(&mods_path) {
                                self.debug_output.clear();
                                self.debug_output.push_str(&format!("[ERROR] Failed to create Mods folder: {}\n", e));
                                return;
                            }
                        }
                        #[cfg(windows)]
                        {
                            let _ = std::process::Command::new("explorer").arg(mods_path).spawn();
                        }
                        #[cfg(target_os = "macos")]
                        {
                            let _ = std::process::Command::new("open").arg(mods_path).spawn();
                        }
                        #[cfg(target_os = "linux")]
                        {
                            let _ = std::process::Command::new("xdg-open").arg(mods_path).spawn();
                        }
                    }
                }
            });
            ui.add_space(16.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id("installed_mods_section", |ui| {
                ui.heading("Installed Mods Folder List:");
                if self.installed_mods.is_empty() {
                    ui.label("(No mods detected)");
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("installed_mods_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for m in &self.installed_mods {
                                ui.label(m);
                            }
                        });
                }
            });
            ui.separator();
            ui.push_id("debug_output_section", |ui| {
                ui.heading("Debug Output:");
                egui::ScrollArea::vertical()
                    .id_source("debug_output_scroll")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.label(&self.debug_output);
                    });
            });
        });
    }
}

impl GuiApp {
    fn update_mod_list(&mut self) {
        if self.win64_dir.is_empty() {
            self.installed_mods.clear();
            return;
        }
        match core::list_installed_mods(&self.win64_dir) {
            Ok(mods) => self.installed_mods = mods,
            Err(e) => {
                self.installed_mods.clear();
                self.debug_output.push_str(&format!("[ERROR] Failed to list mods: {}\n", e));
            }
        }
        // Save cache after mod list update
        self.cache.last_installed_mods = self.installed_mods.clone();
        self.cache.last_win64_dir = self.win64_dir.clone();
        self.cache.last_debug_output = self.debug_output.clone();
        save_cache(&self.cache);
    }
} 