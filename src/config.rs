/**
 * v1.2.0 10/07/2025
 * Author: Marco Maffei
 *
 */

use crate::config_run::ConfigRun;

use once_cell::sync::Lazy;
use std::path::Path;



pub const MENU_START_ROW: u16 = 13;

pub static CONFIG_RUN: Lazy<ConfigRun> = Lazy::new(|| {
    ConfigRun::load(Path::new("./config/config-run.json"))
});