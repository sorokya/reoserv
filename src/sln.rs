use reqwest::Url;

use crate::{SETTINGS, VERSION};

pub async fn ping_sln() {
    let url = match Url::parse_with_params(&format!("{}check", SETTINGS.sln.url),
        &[
            ("software", "REOSERV"),
            ("v", &VERSION),
            ("retry", &(SETTINGS.sln.rate * 60).to_string()),
            ("host", &SETTINGS.sln.hostname),
            ("port", &SETTINGS.server.port),
            ("name", &SETTINGS.sln.server_name),
            ("url", &SETTINGS.sln.site),
            ("zone", &SETTINGS.sln.zone),
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

    debug!("SLN response: {:?}", response);

    if !response.status().is_success() {
        error!("Failed to ping SLN: {} {}", response.status(), response.text().await.unwrap());
        return;
    }

    debug!("Pinged SLN successfully");
}