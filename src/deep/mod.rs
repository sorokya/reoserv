pub const ACTION_CONFIG: u8 = 220;
pub const ACTION_SWAP: u8 = 35;
pub const FAMILY_BOSS: u8 = 52;
// pub const FAMILY_CAPTCHA: u8 = 249;
pub const ACCOUNT_REPLY_WRONG_PIN: i32 = 8;
// pub const AVATAR_CHANGE_TYPE_SKIN: i32 = 4;
// pub const AVATAR_CHANGE_TYPE_GENDER: i32 = 5;

mod account_accept_client_packet;
pub use account_accept_client_packet::AccountAcceptClientPacket;

mod account_accept_server_packet;
pub use account_accept_server_packet::AccountAcceptServerPacket;

mod account_validation_reply;
pub use account_validation_reply::AccountValidationReply;

mod account_recover_reply;
pub use account_recover_reply::AccountRecoverReply;

mod account_recover_pin_reply;
pub use account_recover_pin_reply::AccountRecoverPinReply;

mod account_recover_update_reply;
pub use account_recover_update_reply::AccountRecoverUpdateReply;

mod account_config_server_packet;
pub use account_config_server_packet::AccountConfigServerPacket;

mod boss_ping_server_packet;
pub use boss_ping_server_packet::BossPingServerPacket;

mod captcha_open_server_packet;
pub use captcha_open_server_packet::CaptchaOpenServerPacket;

mod captcha_agree_server_packet;
pub use captcha_agree_server_packet::CaptchaAgreeServerPacket;

mod captcha_close_server_packet;
pub use captcha_close_server_packet::CaptchaCloseServerPacket;

mod captcha_reply_client_packet;
pub use captcha_reply_client_packet::CaptchaReplyClientPacket;

mod captcha_request_client_packet;
pub use captcha_request_client_packet::CaptchaRequestClientPacket;

mod item_report_client_packet;
pub use item_report_client_packet::ItemReportClientPacket;

mod login_config_server_packet;
pub use login_config_server_packet::LoginConfigServerPacket;

mod login_take_client_packet;
pub use login_take_client_packet::LoginTakeClientPacket;

mod login_take_server_packet;
pub use login_take_server_packet::LoginTakeServerPacket;

mod login_create_client_packet;
pub use login_create_client_packet::LoginCreateClientPacket;

mod login_create_server_packet;
pub use login_create_server_packet::LoginCreateServerPacket;

mod login_accept_client_packet;
pub use login_accept_client_packet::LoginAcceptClientPacket;

mod login_accept_server_packet;
pub use login_accept_server_packet::LoginAcceptServerPacket;

mod login_agree_client_packet;
pub use login_agree_client_packet::LoginAgreeClientPacket;

mod login_agree_server_packet;
pub use login_agree_server_packet::LoginAgreeServerPacket;

mod admin_interact_get_client_packet;
pub use admin_interact_get_client_packet::AdminInteractGetClientPacket;

mod admin_interact_create_server_packet;
pub use admin_interact_create_server_packet::AdminInteractCreateServerPacket;

mod admin_interact_add_server_packet;
pub use admin_interact_add_server_packet::AdminInteractAddServerPacket;

mod dialog_line;
pub use dialog_line::DialogLine;

mod lookup_type;
pub use lookup_type::LookupType;

mod paperdoll_swap_server_packet;
pub use paperdoll_swap_server_packet::PaperdollSwapServerPacket;
