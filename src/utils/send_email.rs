use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};

use crate::SETTINGS;

pub async fn send_email(to: &str, to_name: &str, subject: &str, body: &str) -> anyhow::Result<()> {
    let message = MessageBuilder::new()
        .from((
            SETTINGS.smtp.from_name.to_owned(),
            SETTINGS.smtp.from_address.to_owned(),
        ))
        .to((to_name, to))
        .subject(subject)
        .text_body(body);

    SmtpClientBuilder::new(SETTINGS.smtp.host.to_owned(), SETTINGS.smtp.port)
        .implicit_tls(false)
        .credentials((
            SETTINGS.smtp.username.to_owned(),
            SETTINGS.smtp.password.to_owned(),
        ))
        .connect()
        .await?
        .send(message)
        .await?;

    Ok(())
}
