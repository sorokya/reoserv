#[derive(Debug, Serialize)]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub real_name: String,
    pub location: String,
}
