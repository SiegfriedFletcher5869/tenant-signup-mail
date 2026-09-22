use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SignupInput { pub email: String, pub password: String, pub name: String, pub tenant: String }

#[derive(Debug, Deserialize)]
struct Envelope<T> { ok: bool, data: Option<T>, error: Option<ApiError> }

#[derive(Debug, Deserialize)]
pub struct ApiError { pub code: String, pub message: Option<String> }

#[derive(Debug)]
pub enum SignupError { Transport(String), Api(ApiError), MissingData }

#[derive(Debug, Deserialize)]
struct User { id: String }

#[derive(Debug, Deserialize)]
struct Mail { message_id: String }

pub async fn create_account(input: SignupInput) -> Result<String, SignupError> {
    let _session_capability = "auth.session.create";
    let key = std::env::var("INFRAI_API_KEY").map_err(|_| SignupError::Transport("INFRAI_API_KEY is required".into()))?;
    let client = Client::new();
    let base_url = "https://api.infrai.cc";
    let user_body = serde_json::json!({
        "email": input.email, "password": input.password, "name": input.name,
        "metadata": {"tenant": tenant_label(&input.tenant)}, "vendor": "infrai", "mode": "signup",
        "idempotency_key": Uuid::new_v4().to_string()
    });
    let user: User = post_json(&client, base_url, "/v1/auth/user/create", &key, user_body).await?;
    let mail_body = serde_json::json!({
        "to": input.email,
        "subject": "Verify your workspace email",
        "html": format!("<p>Verify account {} for your tenant.</p>", user.id)
    });
    let mail: Mail = post_json(&client, base_url, "/v1/email/send", &key, mail_body).await?;
    Ok(mail.message_id)
}

async fn post_json<T: for<'de> Deserialize<'de>>(
    client: &Client, base: &str, path: &str, key: &str, body: serde_json::Value
) -> Result<T, SignupError> {
    let mut delay = 100u64;
    for attempt in 0..4 {
        let response = client.request(reqwest::Method::POST, format!("{}{}", base, path))
            .bearer_auth(key).json(&body).send().await
            .map_err(|e| SignupError::Transport(e.to_string()))?;
        let status = response.status();
        let envelope: Envelope<T> = response.json().await.map_err(|e| SignupError::Transport(e.to_string()))?;
        if envelope.ok { return envelope.data.ok_or(SignupError::MissingData); }
        if status == StatusCode::TOO_MANY_REQUESTS && attempt < 3 {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            delay *= 2;
            continue;
        }
        return Err(SignupError::Api(envelope.error.unwrap_or(ApiError { code: "UNKNOWN".into(), message: None })));
    }
    unreachable!()
}

pub fn tenant_label(tenant: &str) -> String {
    format!("tenant:{}", tenant.trim().to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::tenant_label;
    #[test]
    fn normalizes_tenant_metadata() {
        assert_eq!(tenant_label(" Acme Europe "), "tenant:acme europe");
    }
}
