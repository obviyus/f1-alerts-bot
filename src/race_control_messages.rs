use crate::models::RaceControlMessages;
use crate::models::SessionInfo;
use chrono::Utc;
use log::info;

const F1_SESSION_ENDPOINT: &str = "https://livetiming.formula1.com/static";

pub async fn is_session_generating() -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::new();

    let body = client
        .get(format!("{}/{}", F1_SESSION_ENDPOINT, "SessionInfo.json"))
        .send()
        .await?
        .text_with_charset("utf-8-sig")
        .await?;

    // Strip BOM from UTF-8-SIG
    let session: SessionInfo = serde_json::from_str(body.trim_start_matches('\u{feff}')).unwrap();

    if session.archive_status.status == "Generating" {
        info!("Session {} is generating...", &session.path);
        Ok(Some(session.path))
    } else {
        Ok(None)
    }
}

pub async fn fetch_latest_race_control_message(
    path: &str,
) -> Result<Option<(String, i64)>, reqwest::Error> {
    let client = reqwest::Client::new();

    let body = client
        .get(format!(
            "{}/{}RaceControlMessages.json",
            F1_SESSION_ENDPOINT, path
        ))
        .send()
        .await?
        .text_with_charset("utf-8-sig")
        .await?;

    // Strip BOM from UTF-8-SIG
    let session: RaceControlMessages =
        serde_json::from_str(body.trim_start_matches('\u{feff}')).unwrap();

    // RaceControlMessages are sorted by timestamp, so the last one is the most recent
    if let Some(message) = session.messages.last() {
        let message_time = chrono::DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::parse_from_str(&message.utc, "%Y-%m-%dT%H:%M:%S").unwrap(),
            Utc,
        );

        return Ok(Some((
            format!("{}: {}", message_time.format("%H:%M:%S"), message.message),
            message_time.timestamp(),
        )));
    }

    Ok(None)
}
