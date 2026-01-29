/**
 * v1.12.0 27/09/2025
 * Author: Marco Maffei
 *
 */

mod config_run;
mod config;
mod app_context;
mod logger;
mod dump; 
mod spinner;
mod options;
mod gui;
mod view_type;
mod tokens;
mod monitor_state;
mod audio_manager;
mod audio_assets;
mod menu_mod;
mod generic_mod;
mod monitor_mod;
mod notification_mod;

use config::CONFIG_RUN;
use options::Options;
use app_context::AppContext;
use colored::control;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    control::set_override(true);

    let options = Options::load("./config", &CONFIG_RUN.config_file_name);
    let mut app_context = AppContext::new(options);

    menu_mod::show_menu(&mut app_context)?;

    Ok(())
}