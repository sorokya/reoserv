use crate::errors::MissingSessionIdError;

use super::Player;

impl Player {
    pub fn take_session_id(&mut self) -> Result<i32, MissingSessionIdError> {
        if let Some(session_id) = self.session_id {
            self.session_id = None;
            Ok(session_id)
        } else {
            Err(MissingSessionIdError)
        }
    }
}
