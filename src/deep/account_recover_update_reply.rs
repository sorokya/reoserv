#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[derive(Default)]
pub enum AccountRecoverUpdateReply {
    #[default]
    Error,
    OK,
    Unrecognized(i32),
}

impl From<i32> for AccountRecoverUpdateReply {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Error,
            1 => Self::OK,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<AccountRecoverUpdateReply> for i32 {
    fn from(value: AccountRecoverUpdateReply) -> i32 {
        match value {
            AccountRecoverUpdateReply::Error => 0,
            AccountRecoverUpdateReply::OK => 1,
            AccountRecoverUpdateReply::Unrecognized(value) => value,
        }
    }
}

