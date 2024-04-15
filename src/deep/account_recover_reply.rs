#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AccountRecoverReply {
    AccountNotFound,
    RequestAccepted,
    RequestAcceptedShowEmail,
    TooManyAttempts,
    RecoveryDisabled,
    Busy,
    TooManyEmails,
    Unrecognized(i32),
}

impl From<i32> for AccountRecoverReply {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::AccountNotFound,
            1 => Self::RequestAccepted,
            2 => Self::RequestAcceptedShowEmail,
            3 => Self::TooManyAttempts,
            4 => Self::RecoveryDisabled,
            5 => Self::Busy,
            6 => Self::TooManyEmails,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<AccountRecoverReply> for i32 {
    fn from(value: AccountRecoverReply) -> i32 {
        match value {
            AccountRecoverReply::AccountNotFound => 0,
            AccountRecoverReply::RequestAccepted => 1,
            AccountRecoverReply::RequestAcceptedShowEmail => 2,
            AccountRecoverReply::TooManyAttempts => 3,
            AccountRecoverReply::RecoveryDisabled => 4,
            AccountRecoverReply::Busy => 5,
            AccountRecoverReply::TooManyEmails => 6,
            AccountRecoverReply::Unrecognized(value) => value,
        }
    }
}

impl Default for AccountRecoverReply {
    fn default() -> Self {
        Self::AccountNotFound
    }
}
