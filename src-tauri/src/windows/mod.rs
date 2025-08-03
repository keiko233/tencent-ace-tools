use crate::windows::ace_tools::AceProcessController;
use std::sync::Mutex;

pub mod ace_tools;
pub mod utils;

// State wrapper for AceProcessController
pub struct AceProcessControllerState(pub Mutex<AceProcessController>);

impl Default for AceProcessControllerState {
    fn default() -> Self {
        Self(Mutex::new(AceProcessController::new()))
    }
}
