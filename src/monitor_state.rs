/**
 * v1.1.0 18/07/2025
 * Author: Marco Maffei
 *
 */

pub struct MonitorState {
    pub is_first_output: bool,
    pub is_paused: bool,
    pub prev_value_f: Option<f32>,
    pub total_perc_diff: f32,
    pub min_price_reached: f32,
    pub max_price_reached: f32,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            is_first_output: true,
            is_paused: false,
            prev_value_f: None,
            total_perc_diff: 0.0,
            min_price_reached: 0.0,
            max_price_reached: 0.0,
        }
    }
}