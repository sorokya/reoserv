#[derive(Debug, Default)]
pub struct Captcha {
    pub challenge: String,
    pub reward: i32,
    pub attempts: i32,
}
