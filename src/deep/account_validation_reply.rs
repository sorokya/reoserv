#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AccountValidationReply {
    Busy,
    OK,
    TooManyAttempts,
    Unrecognized(i32),
}

impl From<i32> for AccountValidationReply {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Busy,
            1 => Self::OK,
            2 => Self::TooManyAttempts,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<AccountValidationReply> for i32 {
    fn from(value: AccountValidationReply) -> i32 {
        match value {
            AccountValidationReply::Busy => 0,
            AccountValidationReply::OK => 1,
            AccountValidationReply::TooManyAttempts => 2,
            AccountValidationReply::Unrecognized(value) => value,
        }
    }
}

impl Default for AccountValidationReply {
    fn default() -> Self {
        Self::Busy
    }
}
