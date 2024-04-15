#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AccountRecoverPinReply {
    WrongPin,
    OK,
    Unrecognized(i32),
}

impl From<i32> for AccountRecoverPinReply {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::WrongPin,
            1 => Self::OK,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<AccountRecoverPinReply> for i32 {
    fn from(value: AccountRecoverPinReply) -> i32 {
        match value {
            AccountRecoverPinReply::WrongPin => 0,
            AccountRecoverPinReply::OK => 1,
            AccountRecoverPinReply::Unrecognized(value) => value,
        }
    }
}

impl Default for AccountRecoverPinReply {
    fn default() -> Self {
        Self::WrongPin
    }
}
