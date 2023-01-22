use reqwest::Url;

use crate::{SETTINGS, VERSION};

pub async fn ping_sln() {
    let version_parts = SETTINGS.server.max_version.split('.').collect::<Vec<&str>>();
    let client_major_version = version_parts[0];
    let client_minor_version = version_parts[1];
    let url = match Url::parse_with_params(&format!("{}check", SETTINGS.sln.url),
        &[
            ("software", "REOSERV"),
            ("v", VERSION),
            ("retry", &(SETTINGS.sln.rate * 60).to_string()),
            ("host", &SETTINGS.sln.hostname),
            ("port", &SETTINGS.server.port),
            ("name", &SETTINGS.sln.server_name),
            ("url", &SETTINGS.sln.site),
            ("zone", &SETTINGS.sln.zone),
            ("clientmajorversion", client_major_version),
            ("clientminorversion", client_minor_version),
        ],
    ) {
        Ok(url) => url,
        Err(e) => {
            error!("Failed to parse SLN url: {}", e);
            return;
        }
    };

    debug!("Pinging SLN: {}", url);

    let client = reqwest::Client::new();
    let response = match client.get(url)
        .header("User-Agent", "EOSERV")
        .send()
        .await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to ping SLN: {}", e);
                return;
            }
    };

    if !response.status().is_success() {
        error!("Failed to ping SLN: {} {}", response.status(), response.text().await.unwrap());
        return;
    }

    if let Ok(message) = response.text().await {
        let lines = message.split('\n').collect::<Vec<&str>>();
        for line in lines {
            let code = match line.chars().next() {
                Some(code) => code as u32,
                None => continue,
            };

            match code {
                3 | 4 | 5 => warn!("SLN Error: {}", line),
                _ => continue,
            }
        }
    }

    debug!("Pinged SLN successfully");
}