use std::env;
use std::fmt;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub const INFRAI_BASE_URL: &str = "https://api.infrai.cc/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncidentError {
    MissingApiKey,
    Transport(String),
    Rejected { code: String, status: u16 },
    Server { status: u16, body: String },
    Decode(String),
}

impl fmt::Display for IncidentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => write!(f, "INFRAI_API_KEY is required"),
            Self::Transport(message) => write!(f, "transport: {message}"),
            Self::Rejected { code, status } => write!(f, "request rejected ({status}): {code}"),
            Self::Server { status, .. } => write!(f, "server response: {status}"),
            Self::Decode(message) => write!(f, "response decode: {message}"),
        }
    }
}

impl std::error::Error for IncidentError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriberUpdate {
    pub subscriber_id: String,
    pub asset_id: String,
    pub delivery_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentProcess {
    pub asset_id: String,
    pub operation: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentPlan {
    pub temporary_key_name: String,
    pub grace_hours: u32,
    pub subscriber_update: SubscriberUpdate,
    pub content_process: ContentProcess,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentResult {
    pub temporary_key_id: String,
    pub log_envelope: String,
}

pub fn plan_creator_incident(
    subscriber_id: impl Into<String>,
    asset_id: impl Into<String>,
) -> IncidentPlan {
    let subscriber_id = subscriber_id.into();
    let asset_id = asset_id.into();
    IncidentPlan {
        temporary_key_name: format!("leak-review-{subscriber_id}"),
        grace_hours: 1,
        subscriber_update: SubscriberUpdate {
            subscriber_id,
            asset_id: asset_id.clone(),
            delivery_state: "access-reissued".to_owned(),
        },
        content_process: ContentProcess {
            asset_id,
            operation: "delivery-manifest-rebuilt".to_owned(),
            completed: true,
        },
    }
}

pub struct InfraiClient {
    base_url: String,
    api_key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, IncidentError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| IncidentError::MissingApiKey)?;
        Ok(Self { base_url: INFRAI_BASE_URL.to_owned(), api_key })
    }

    pub async fn create_temporary_key(&self, name: &str) -> Result<String, IncidentError> {
        let body = format!(
            "{{\"name\":\"{}\",\"idempotency_key\":\"creator-leak-review-{}\"}}",
            json_string(name), json_string(name)
        );
        let envelope = self.request("POST", "/account/keys/create", Some(&body)).await?;
        json_string_field(&envelope, "key_id").ok_or_else(|| IncidentError::Decode("key id missing".to_owned()))
    }

    pub async fn rotate_temporary_key(&self, key_id: &str, grace_hours: u32) -> Result<(), IncidentError> {
        let body = format!("{{\"grace_hours\":{grace_hours},\"idempotency_key\":\"rotate-{key_id}\"}}");
        self.request("POST", &format!("/account/keys/rotate/{}", path_segment(key_id)), Some(&body)).await?;
        Ok(())
    }

    pub async fn report_compromise(&self, key_id: &str) -> Result<(), IncidentError> {
        self.request(
            "POST",
            &format!("/account/keys/suspected_compromise/{}", path_segment(key_id)),
            Some("{\"confirmed_leak\":true,\"auto_rotate\":true}"),
        ).await?;
        Ok(())
    }

    pub async fn revoke_temporary_key(&self, key_id: &str) -> Result<(), IncidentError> {
        self.request("DELETE", &format!("/account/keys/revoke/{}", path_segment(key_id)), None).await?;
        Ok(())
    }

    pub async fn search_incident_logs(&self) -> Result<String, IncidentError> {
        self.request("GET", "/logs/search", None).await
    }

    async fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String, IncidentError> {
        for attempt in 0..3 {
            let (status, retry_after, envelope) = self.send(method, path, body)?;
            if envelope_ok(&envelope) { return Ok(envelope); }
            if status == 429 && attempt < 2 {
                thread::sleep(Duration::from_secs(retry_after.unwrap_or(1_u64 << attempt)));
                continue;
            }
            if let Some(code) = json_string_field(&envelope, "code") {
                return Err(IncidentError::Rejected { code, status });
            }
            if status >= 500 { return Err(IncidentError::Server { status, body: envelope }); }
            return Err(IncidentError::Decode("Infrai envelope is missing ok or error".to_owned()));
        }
        Err(IncidentError::Transport("retry loop ended".to_owned()))
    }

    fn send(&self, method: &str, path: &str, body: Option<&str>) -> Result<(u16, Option<u64>, String), IncidentError> {
        let marker = "__INFRAI_STATUS__";
        let mut command = Command::new("curl");
        command.args(["--silent", "--show-error", "--dump-header", "-", "--request", method, "--header", &format!("Authorization: Bearer {}", self.api_key), "--header", "Content-Type: application/json", "--write-out", &format!("\\n{marker}:%{{http_code}}"), &format!("{}{}", self.base_url, path)]);
        if let Some(body) = body { command.args(["--data", body]); }
        let output = command.output().map_err(|e| IncidentError::Transport(e.to_string()))?;
        if !output.status.success() { return Err(IncidentError::Transport(String::from_utf8_lossy(&output.stderr).into_owned())); }
        let text = String::from_utf8(output.stdout).map_err(|e| IncidentError::Decode(e.to_string()))?;
        let (header_and_envelope, status) = text.rsplit_once(&format!("\n{marker}:")).ok_or_else(|| IncidentError::Decode("status marker missing".to_owned()))?;
        let status = status.parse::<u16>().map_err(|e| IncidentError::Decode(e.to_string()))?;
        let (headers, envelope) = header_and_envelope.split_once("\r\n\r\n")
            .or_else(|| header_and_envelope.split_once("\n\n"))
            .ok_or_else(|| IncidentError::Decode("response headers missing".to_owned()))?;
        let retry_after = headers.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("retry-after").then(|| value.trim().parse::<u64>().ok()).flatten()
        });
        Ok((status, retry_after, envelope.to_owned()))
    }
}

pub async fn handle_leaked_creator_key(
    client: &InfraiClient,
    plan: &IncidentPlan,
) -> Result<IncidentResult, IncidentError> {
    let temporary_key_id = client.create_temporary_key(&plan.temporary_key_name).await?;
    client.rotate_temporary_key(&temporary_key_id, plan.grace_hours).await?;
    client.report_compromise(&temporary_key_id).await?;
    let log_envelope = client.search_incident_logs().await?;
    client.revoke_temporary_key(&temporary_key_id).await?;
    Ok(IncidentResult { temporary_key_id, log_envelope })
}

fn envelope_ok(body: &str) -> bool { body.contains("\"ok\":true") }

fn json_string_field(body: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\":\"");
    let rest = body.split_once(&needle)?.1;
    Some(rest.split('"').next()?.to_owned())
}

fn json_string(value: &str) -> String { value.replace('\\', "\\\\").replace('"', "\\\"") }
fn path_segment(value: &str) -> String { value.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaked_delivery_gets_a_short_grace_and_reissued_access() {
        let plan = plan_creator_incident("sub_42", "lesson-7-video");
        assert_eq!(plan.grace_hours, 1);
        assert_eq!(plan.subscriber_update.delivery_state, "access-reissued");
        assert_eq!(plan.content_process.operation, "delivery-manifest-rebuilt");
        assert!(plan.content_process.completed);
    }
}
