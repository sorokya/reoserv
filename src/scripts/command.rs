use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command {
    Tick,
    HandlePlayerCommand {
        args: Vec<String>,
        respond_to: oneshot::Sender<bool>,
    },
}
