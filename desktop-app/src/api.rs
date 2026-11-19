use crate::model::{
    Currency, SyncPayload, SyncResponse, User, default_currency_code, normalize_currency_code,
};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("failed to start the HTTP runtime")
    })
}

#[derive(Clone, Debug)]
pub struct ApiError {
    pub message: String,
    pub status: Option<u16>,
}

impl ApiError {
    fn network(base_url: &str, error: impl fmt::Display) -> Self {
        Self {
            message: format!(
                "Couldn't reach the Sunder server at {base_url}. Check that it's running and that you're online.\n{error}"
            ),
            status: None,
        }
    }

    fn from_response(status: u16, body: &str) -> Self {
        let parsed: Option<Value> = serde_json::from_str(body).ok();
        let json = parsed.unwrap_or(Value::Null);

        let message = json
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                json.get("error")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .or_else(|| {
                json.get("errors").and_then(Value::as_object).map(|errors| {
                    errors
                        .iter()
                        .map(|(field, value)| {
                            let detail = value
                                .as_array()
                                .map(|items| {
                                    items
                                        .iter()
                                        .filter_map(Value::as_str)
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .unwrap_or_default();
                            if detail.is_empty() {
                                field.clone()
                            } else {
                                format!("{field}: {detail}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
            })
            .unwrap_or_else(|| default_message(status).to_string());

        Self {
            message,
            status: Some(status),
        }
    }
}

fn default_message(status: u16) -> &'static str {
    match status {
        400 => "The server rejected that request.",
        401 => "Your session has expired. Please sign in again.",
        403 => "You don't have permission to do that.",
        404 => "The server couldn't find what you asked for.",
        409 => "That change conflicts with something else.",
        422 => "Some of those values aren't valid.",
        500..=599 => "The Sunder server ran into a problem. Try again in a moment.",
        _ => "The request failed.",
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(default)]
    pub refresh_token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RefreshResponse {
    #[serde(default)]
    pub access_token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileResponse {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub flags: Vec<String>,
    #[serde(default = "default_currency_code")]
    pub default_currency_code: String,
}

impl ProfileResponse {
    pub fn into_user(self, email: String) -> User {
        User {
            username: self.username,
            email: if self.email.is_empty() {
                email
            } else {
                self.email
            },
            flags: self.flags,
            default_currency_code: normalize_currency_code(&self.default_currency_code),
        }
    }
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        let base_url = crate::store::normalize_base_url(&base_url);
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(10))
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build the HTTP client");

        Self { base_url, client }
    }

    pub async fn login(&self, login: String, pass: String) -> Result<LoginResponse, ApiError> {
        let url = format!("{}/auth/login", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .post(url)
                .json(&serde_json::json!({ "login": login, "pass": pass }))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn register(
        &self,
        invite: String,
        email: String,
        username: String,
        pass: String,
    ) -> Result<(), ApiError> {
        let url = format!("{}/auth/register/{}", self.base_url, urlencode(&invite));
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .post(url)
                .json(&serde_json::json!({
                    "email": email,
                    "username": username,
                    "pass": pass
                }))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let _: serde_json::Value = decode(response, &base_url).await?;
            Ok(())
        })
        .await
    }

    pub async fn refresh(&self, token: String) -> Result<RefreshResponse, ApiError> {
        let url = format!("{}/auth/refresh_token", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .post(url)
                .json(&serde_json::json!({ "token": token }))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn profile(&self, access_token: String) -> Result<ProfileResponse, ApiError> {
        let url = format!("{}/dashboard/", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn sync(
        &self,
        access_token: String,
        last: Option<String>,
        payload: SyncPayload,
    ) -> Result<SyncResponse, ApiError> {
        let url = format!("{}/dashboard/eco/sync", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        let last = last.unwrap_or_else(|| "1970-01-01T00:00:00".to_string());
        self.run(async move {
            let response = client
                .post(url)
                .bearer_auth(access_token)
                .query(&[("last", last)])
                .json(&payload)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    async fn run<T, F>(&self, future: F) -> Result<T, ApiError>
    where
        T: Send + 'static,
        F: std::future::Future<Output = Result<T, ApiError>> + Send + 'static,
    {
        runtime().spawn(future).await.map_err(|error| ApiError {
            message: format!("The request was interrupted.\n{error}"),
            status: None,
        })?
    }
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

async fn decode<T>(response: reqwest::Response, base_url: &str) -> Result<T, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status().as_u16();
    let body = response.text().await.map_err(|error| ApiError {
        message: format!("Couldn't read the server's response.\n{error}"),
        status: Some(status),
    })?;

    if !(200..300).contains(&status) {
        return Err(ApiError::from_response(status, &body));
    }

    serde_json::from_str(&body).map_err(|error| ApiError {
        message: format!(
            "The server sent a response Sunder couldn't understand.\n{error}\n{base_url}"
        ),
        status: Some(status),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

    fn mock_server(
        respond: impl FnOnce(&str) -> (u16, &'static str) + Send + 'static,
    ) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = socket.read(&mut buffer).unwrap_or(0);
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                let text = String::from_utf8_lossy(&request);
                if let Some(header_end) = text.find("\r\n\r\n") {
                    let content_length = text[..header_end]
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .unwrap_or(0);
                    if request.len() >= header_end + 4 + content_length {
                        break;
                    }
                }
            }
            let request = String::from_utf8_lossy(&request).to_string();
            let (status, body) = respond(&request);
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).unwrap();
        });
        (format!("http://{address}"), handle)
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    #[test]
    fn encodes_invite_codes_for_the_path() {
        assert_eq!(urlencode("abc123"), "abc123");
        assert_eq!(urlencode("a b"), "a%20b");
        assert_eq!(urlencode("a/b?c"), "a%2Fb%3Fc");
    }

    #[test]
    fn login_posts_credentials_and_reads_the_refresh_token() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("POST /auth/login "));
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("content-type: application/json")
            );
            assert!(request.contains(r#"{"login":"me@example.com","pass":"hunter2"}"#));
            (200, r#"{"refresh_token":"refresh-123","message":"ok"}"#)
        });
        let client = ApiClient::new(base);
        let response = block_on(client.login("me@example.com".into(), "hunter2".into())).unwrap();
        assert_eq!(response.refresh_token, "refresh-123");
        server.join().unwrap();
    }

    #[test]
    fn sync_sends_the_cursor_and_bearer_token() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("POST /dashboard/eco/sync?last=2026-01-01T00"));
            assert!(request.contains("authorization: Bearer access-123"));
            assert!(request.contains(r#""accounts":[]"#));
            assert!(request.contains(r#""entries":[]"#));
            (
                200,
                r#"{"accounts":[],"categories":[],"entries":[{"id":"e1","title":"Coffee","amount":-4.5,"date":"2026-01-01T12:00:00","account_id":"a","category_id":"c","inserted_at":"2026-01-01T12:00:00","updated_at":"2026-01-01T12:00:00"}]}"#,
            )
        });
        let client = ApiClient::new(base);
        let response = block_on(client.sync(
            "access-123".into(),
            Some("2026-01-01T00:00:00".into()),
            SyncPayload::default(),
        ))
        .unwrap();
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].title, "Coffee");
        server.join().unwrap();
    }

    #[test]
    fn admin_endpoints_use_the_expected_contract() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /admin/users "));
            assert!(request.contains("authorization: Bearer access-1"));
            (
                200,
                r#"{"users":[{"id":"u1","username":"alice","email":"a@example.com","flags":["is_admin"]}]}"#,
            )
        });
        let client = ApiClient::new(base);
        let users = block_on(client.admin_users("access-1".into())).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "alice");
        assert!(users[0].flags.contains(&"is_admin".to_string()));
        server.join().unwrap();
    }

    #[test]
    fn admin_users_tolerates_null() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /admin/users "));
            (200, r#"{"users":null}"#)
        });
        let client = ApiClient::new(base);
        assert!(
            block_on(client.admin_users("access-1".into()))
                .unwrap()
                .is_empty()
        );
        server.join().unwrap();
    }

    #[test]
    fn admin_user_flags_tolerate_null() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /admin/users "));
            (
                200,
                r#"{"users":[{"id":"u1","username":"alice","email":"a@example.com","flags":null}]}"#,
            )
        });
        let client = ApiClient::new(base);
        let users = block_on(client.admin_users("access-1".into())).unwrap();
        assert_eq!(users.len(), 1);
        assert!(users[0].flags.is_empty());
        server.join().unwrap();
    }

    #[test]
    fn admin_invites_tolerates_null() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /admin/invites "));
            (200, r#"{"invites":null}"#)
        });
        let client = ApiClient::new(base);
        assert!(
            block_on(client.admin_invites("access-1".into()))
                .unwrap()
                .is_empty()
        );
        server.join().unwrap();
    }

    #[test]
    fn export_streams_json_bytes() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /dashboard/eco/data/export "));
            assert!(request.contains("authorization: Bearer access-1"));
            (200, r#"{"format":"sunder","accounts":[]}"#)
        });
        let client = ApiClient::new(base);
        let bytes = block_on(client.export_data("access-1".into())).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("\"sunder\""));
        server.join().unwrap();
    }

    #[test]
    fn imports_use_multipart_and_read_stats() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("POST /dashboard/eco/data/import/sunder "));
            assert!(request.contains("authorization: Bearer access-1"));
            assert!(request.to_ascii_lowercase().contains("multipart/form-data"));
            assert!(request.contains("name=\"file\""));
            assert!(request.contains("filename=\"backup.json\""));
            assert!(request.contains("{\"format\":\"sunder\"}"));
            (
                200,
                r#"{"success":true,"stats":{"accounts":1,"categories":2,"entries":3}}"#,
            )
        });
        let client = ApiClient::new(base);
        let stats = block_on(client.import_data(
            "access-1".into(),
            ImportKind::Sunder,
            "backup.json".into(),
            br#"{"format":"sunder"}"#.to_vec(),
        ))
        .unwrap();
        assert_eq!(stats.accounts, 1);
        assert_eq!(stats.categories, 2);
        assert_eq!(stats.entries, 3);
        server.join().unwrap();
    }

    #[test]
    fn admin_user_updates_carry_flags_and_passwords() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("PUT /admin/users/u1 "));
            assert!(request.contains(r#""flags":["is_admin"]"#));
            assert!(request.contains(r#""pass":"secret""#));
            (
                200,
                r#"{"success":true,"user":{"id":"u1","username":"alice","email":"new@example.com","flags":["is_admin"]}}"#,
            )
        });
        let client = ApiClient::new(base);
        let user = block_on(client.admin_update_user(
            "access-1".into(),
            "u1".into(),
            "alice".into(),
            "new@example.com".into(),
            vec!["is_admin".into()],
            Some("secret".into()),
        ))
        .unwrap();
        assert_eq!(user.email, "new@example.com");
        server.join().unwrap();
    }

    #[test]
    fn server_errors_become_friendly_api_errors() {
        let (base, server) = mock_server(|_| {
            (
                400,
                r#"{"error_code":"INVALID_CREDENTIALS","message":"Invalid email or password."}"#,
            )
        });
        let client = ApiClient::new(base);
        let error = block_on(client.login("me".into(), "wrong".into())).unwrap_err();
        assert_eq!(error.status, Some(400));
        assert_eq!(error.message, "Invalid email or password.");
        server.join().unwrap();
    }

    #[test]
    fn lists_server_allowed_currencies() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /currencies "));
            (
                200,
                r#"{"currencies":[{"code":"USD","name":"US Dollar","symbol":"$","symbol_after":false,"decimal_digits":2},{"code":"PLN","name":"Polish Zloty","symbol":"zł","symbol_after":true,"decimal_digits":2}]}"#,
            )
        });
        let client = ApiClient::new(base);
        let currencies = block_on(client.list_currencies()).unwrap();
        assert_eq!(currencies.len(), 2);
        assert_eq!(currencies[1].code, "PLN");
        assert!(currencies[1].symbol_after);
        server.join().unwrap();
    }

    #[test]
    fn fetches_rates_history_and_updates_preferences() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /currencies/rates/history?from=2026-01-01&base=USD&to=2026-01-10 "));
            (
                200,
                r#"{"base":"USD","from":"2026-01-01","to":"2026-01-10","rates":{"2026-01-01":{"EUR":0.85,"USD":1.0}}}"#,
            )
        });
        let client = ApiClient::new(base);
        let history = block_on(client.rates_history(
            Some("USD".into()),
            "2026-01-01".into(),
            Some("2026-01-10".into()),
        ))
        .unwrap();
        assert_eq!(history.rates["2026-01-01"]["EUR"], 0.85);
        server.join().unwrap();

        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("PUT /dashboard/eco/preferences "));
            assert!(request.contains("authorization: Bearer access-1"));
            assert!(request.contains(r#""default_currency_code":"EUR""#));
            (200, r#"{"default_currency_code":"EUR","default_currency":{"code":"EUR","name":"Euro","symbol":"€","symbol_after":false,"decimal_digits":2}}"#)
        });
        let client = ApiClient::new(base);
        let prefs = block_on(client.update_preferences("access-1".into(), "eur".into())).unwrap();
        assert_eq!(prefs.default_currency_code, "EUR");
        server.join().unwrap();
    }

    #[test]
    fn admin_currency_and_rate_endpoints_match_contract() {
        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("GET /admin/currencies "));
            assert!(request.contains("authorization: Bearer access-1"));
            (
                200,
                r#"{"currencies":[{"code":"USD","name":"US Dollar","symbol":"$","symbol_after":false,"decimal_digits":2,"is_active":true}]}"#,
            )
        });
        let client = ApiClient::new(base);
        let list = block_on(client.admin_currencies("access-1".into())).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].is_active);
        server.join().unwrap();

        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("PUT /admin/currencies/EUR "));
            assert!(request.contains(r#""is_active":false"#));
            (
                200,
                r#"{"success":true,"currency":{"code":"EUR","name":"Euro","symbol":"€","symbol_after":false,"decimal_digits":2,"is_active":false}}"#,
            )
        });
        let client = ApiClient::new(base);
        let updated = block_on(client.admin_update_currency(
            "access-1".into(),
            "eur".into(),
            AdminCurrencyPatch {
                is_active: Some(false),
                ..Default::default()
            },
        ))
        .unwrap();
        assert!(!updated.is_active);
        server.join().unwrap();

        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("POST /admin/currencies/sync-rates?days=7&force=true "));
            (
                200,
                r#"{"success":true,"from":"2026-01-01","to":"2026-01-07","stats":{"total":7,"synced":7,"skipped":0,"failed":0}}"#,
            )
        });
        let client = ApiClient::new(base);
        let sync_res = block_on(client.admin_sync_rates(
            "access-1".into(),
            None,
            Some(7),
            true,
        ))
        .unwrap();
        assert!(sync_res.success);
        assert_eq!(sync_res.stats.unwrap().synced, 7);
        server.join().unwrap();

        let (base, server) = mock_server(|request| {
            assert!(request.starts_with("PUT /admin/currencies/rates "));
            assert!(request.contains(r#""target_currency_code":"PLN""#));
            assert!(request.contains(r#""rate":4.05"#));
            (
                200,
                r#"{"success":true,"rate":{"base_currency_code":"USD","target_currency_code":"PLN","rate":4.05,"rate_date":"2026-01-07","source":"manual"}}"#,
            )
        });
        let client = ApiClient::new(base);
        let manual = block_on(client.admin_upsert_rate(
            "access-1".into(),
            None,
            "pln".into(),
            4.05,
            Some("2026-01-07".into()),
        ))
        .unwrap();
        assert_eq!(manual.target_currency_code, "PLN");
        assert_eq!(manual.rate, 4.05);
        server.join().unwrap();
    }
}

/// Treat an explicit JSON `null` like a missing field for collection fields.
fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdminUser {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub flags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdminInvite {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub used_by: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ImportStats {
    #[serde(default)]
    pub accounts: usize,
    #[serde(default)]
    pub categories: usize,
    #[serde(default)]
    pub entries: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportKind {
    Sunder,
    Bagels,
}

impl ImportKind {
    fn endpoint(self) -> &'static str {
        match self {
            ImportKind::Sunder => "sunder",
            ImportKind::Bagels => "bagels",
        }
    }
}

#[derive(Deserialize)]
struct UsersResponse {
    #[serde(default)]
    users: Option<Vec<AdminUser>>,
}

#[derive(Deserialize)]
struct InvitesResponse {
    #[serde(default)]
    invites: Option<Vec<AdminInvite>>,
}

#[derive(Deserialize)]
struct UpdateUserResponse {
    #[serde(default)]
    user: Option<AdminUser>,
}

#[derive(Deserialize)]
struct ImportResponse {
    #[serde(default)]
    stats: ImportStats,
}

impl ApiClient {
    pub async fn admin_users(&self, access_token: String) -> Result<Vec<AdminUser>, ApiError> {
        let url = format!("{}/admin/users", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: UsersResponse = decode(response, &base_url).await?;
            Ok(payload.users.unwrap_or_default())
        })
        .await
    }

    pub async fn admin_invites(&self, access_token: String) -> Result<Vec<AdminInvite>, ApiError> {
        let url = format!("{}/admin/invites", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: InvitesResponse = decode(response, &base_url).await?;
            Ok(payload.invites.unwrap_or_default())
        })
        .await
    }

    pub async fn admin_create_invite(
        &self,
        access_token: String,
        token: String,
    ) -> Result<AdminInvite, ApiError> {
        let url = format!("{}/admin/invites", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .post(url)
                .bearer_auth(access_token)
                .json(&serde_json::json!({ "token": token }))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn admin_delete_invite(
        &self,
        access_token: String,
        id: String,
    ) -> Result<(), ApiError> {
        let url = format!("{}/admin/invites/{}", self.base_url, urlencode(&id));
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .delete(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let _: serde_json::Value = decode(response, &base_url).await?;
            Ok(())
        })
        .await
    }

    pub async fn admin_update_user(
        &self,
        access_token: String,
        id: String,
        username: String,
        email: String,
        flags: Vec<String>,
        pass: Option<String>,
    ) -> Result<AdminUser, ApiError> {
        let url = format!("{}/admin/users/{}", self.base_url, urlencode(&id));
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let mut payload = serde_json::json!({
                "username": username,
                "email": email,
                "flags": flags,
            });
            if let Some(pass) = pass.filter(|pass| !pass.is_empty()) {
                payload["pass"] = serde_json::Value::String(pass);
            }
            let response = client
                .put(url)
                .bearer_auth(access_token)
                .json(&payload)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: UpdateUserResponse = decode(response, &base_url).await?;
            payload.user.ok_or_else(|| ApiError {
                message: "The server updated the user but returned no record.".into(),
                status: Some(500),
            })
        })
        .await
    }

    pub async fn admin_delete_user(
        &self,
        access_token: String,
        id: String,
    ) -> Result<(), ApiError> {
        let url = format!("{}/admin/users/{}", self.base_url, urlencode(&id));
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .delete(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let _: serde_json::Value = decode(response, &base_url).await?;
            Ok(())
        })
        .await
    }

    pub async fn export_data(&self, access_token: String) -> Result<Vec<u8>, ApiError> {
        let url = format!("{}/dashboard/eco/data/export", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let status = response.status().as_u16();
            let bytes = response.bytes().await.map_err(|error| ApiError {
                message: format!("Couldn't read the export.\n{error}"),
                status: Some(status),
            })?;
            if !(200..300).contains(&status) {
                return Err(ApiError::from_response(
                    status,
                    &String::from_utf8_lossy(&bytes),
                ));
            }
            Ok(bytes.to_vec())
        })
        .await
    }

    pub async fn import_data(
        &self,
        access_token: String,
        kind: ImportKind,
        file_name: String,
        bytes: Vec<u8>,
    ) -> Result<ImportStats, ApiError> {
        let url = format!(
            "{}/dashboard/eco/data/import/{}",
            self.base_url,
            kind.endpoint()
        );
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
            let form = reqwest::multipart::Form::new().part("file", part);
            let response = client
                .post(url)
                .bearer_auth(access_token)
                .multipart(form)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: ImportResponse = decode(response, &base_url).await?;
            Ok(payload.stats)
        })
        .await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RatesSnapshotResponse {
    #[serde(default = "default_currency_code")]
    pub base: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub requested_date: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub rates: HashMap<String, f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RatesHistoryResponse {
    #[serde(default = "default_currency_code")]
    pub base: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub rates: HashMap<String, HashMap<String, f64>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PreferencesResponse {
    #[serde(default = "default_currency_code")]
    pub default_currency_code: String,
    #[serde(default)]
    pub default_currency: Option<Currency>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AdminBackfillStats {
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub synced: usize,
    #[serde(default)]
    pub skipped: usize,
    #[serde(default)]
    pub failed: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AdminSyncRatesResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub fetched_date: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub stored_count: Option<usize>,
    #[serde(default)]
    pub stats: Option<AdminBackfillStats>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdminManualRate {
    #[serde(default = "default_currency_code")]
    pub base_currency_code: String,
    #[serde(default)]
    pub target_currency_code: String,
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub rate_date: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Deserialize)]
struct CurrenciesResponse {
    #[serde(default)]
    currencies: Option<Vec<Currency>>,
}

#[derive(Deserialize)]
struct CurrencyMutationResponse {
    #[serde(default)]
    currency: Option<Currency>,
}

#[derive(Deserialize)]
struct UpsertRateResponse {
    #[serde(default)]
    rate: Option<AdminManualRate>,
}

#[derive(Clone, Debug, Default)]
pub struct AdminCurrencyPatch {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub symbol_after: Option<bool>,
    pub decimal_digits: Option<u8>,
    pub is_active: Option<bool>,
}

impl ApiClient {
    pub async fn list_currencies(&self) -> Result<Vec<Currency>, ApiError> {
        let url = format!("{}/currencies", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: CurrenciesResponse = decode(response, &base_url).await?;
            Ok(payload.currencies.unwrap_or_default())
        })
        .await
    }

    pub async fn rates(
        &self,
        base: Option<String>,
        date: Option<String>,
    ) -> Result<RatesSnapshotResponse, ApiError> {
        let url = format!("{}/currencies/rates", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(base) = base.filter(|b| !b.trim().is_empty()) {
                query.push(("base", normalize_currency_code(&base)));
            }
            if let Some(date) = date.filter(|d| !d.trim().is_empty()) {
                query.push(("date", date));
            }
            let response = client
                .get(url)
                .query(&query)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn rates_history(
        &self,
        base: Option<String>,
        from: String,
        to: Option<String>,
    ) -> Result<RatesHistoryResponse, ApiError> {
        let url = format!("{}/currencies/rates/history", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let mut query: Vec<(&str, String)> = vec![("from", from)];
            if let Some(base) = base.filter(|b| !b.trim().is_empty()) {
                query.push(("base", normalize_currency_code(&base)));
            }
            if let Some(to) = to.filter(|t| !t.trim().is_empty()) {
                query.push(("to", to));
            }
            let response = client
                .get(url)
                .query(&query)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn get_preferences(
        &self,
        access_token: String,
    ) -> Result<PreferencesResponse, ApiError> {
        let url = format!("{}/dashboard/eco/preferences", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn update_preferences(
        &self,
        access_token: String,
        default_currency_code: String,
    ) -> Result<PreferencesResponse, ApiError> {
        let url = format!("{}/dashboard/eco/preferences", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        let code = normalize_currency_code(&default_currency_code);
        self.run(async move {
            let response = client
                .put(url)
                .bearer_auth(access_token)
                .json(&serde_json::json!({ "default_currency_code": code }))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn admin_currencies(
        &self,
        access_token: String,
    ) -> Result<Vec<Currency>, ApiError> {
        let url = format!("{}/admin/currencies", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: CurrenciesResponse = decode(response, &base_url).await?;
            Ok(payload.currencies.unwrap_or_default())
        })
        .await
    }

    pub async fn admin_create_currency(
        &self,
        access_token: String,
        currency: Currency,
    ) -> Result<Currency, ApiError> {
        let url = format!("{}/admin/currencies", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let response = client
                .post(url)
                .bearer_auth(access_token)
                .json(&serde_json::json!({
                    "code": normalize_currency_code(&currency.code),
                    "name": currency.name,
                    "symbol": currency.symbol,
                    "symbol_after": currency.symbol_after,
                    "decimal_digits": currency.decimal_digits,
                    "is_active": currency.is_active,
                }))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: CurrencyMutationResponse = decode(response, &base_url).await?;
            payload.currency.ok_or_else(|| ApiError {
                message: "The server created the currency but returned no record.".into(),
                status: Some(500),
            })
        })
        .await
    }

    pub async fn admin_update_currency(
        &self,
        access_token: String,
        code: String,
        patch: AdminCurrencyPatch,
    ) -> Result<Currency, ApiError> {
        let code = normalize_currency_code(&code);
        let url = format!("{}/admin/currencies/{}", self.base_url, urlencode(&code));
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let mut body = serde_json::Map::new();
            if let Some(name) = patch.name {
                body.insert("name".into(), Value::String(name));
            }
            if let Some(symbol) = patch.symbol {
                body.insert("symbol".into(), Value::String(symbol));
            }
            if let Some(symbol_after) = patch.symbol_after {
                body.insert("symbol_after".into(), Value::Bool(symbol_after));
            }
            if let Some(decimal_digits) = patch.decimal_digits {
                body.insert("decimal_digits".into(), Value::from(decimal_digits));
            }
            if let Some(is_active) = patch.is_active {
                body.insert("is_active".into(), Value::Bool(is_active));
            }
            let response = client
                .put(url)
                .bearer_auth(access_token)
                .json(&Value::Object(body))
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: CurrencyMutationResponse = decode(response, &base_url).await?;
            payload.currency.ok_or_else(|| ApiError {
                message: "The server updated the currency but returned no record.".into(),
                status: Some(500),
            })
        })
        .await
    }

    pub async fn admin_sync_rates(
        &self,
        access_token: String,
        date: Option<String>,
        days: Option<u32>,
        force: bool,
    ) -> Result<AdminSyncRatesResponse, ApiError> {
        let url = format!("{}/admin/currencies/sync-rates", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(date) = date.filter(|d| !d.trim().is_empty()) {
                query.push(("date", date));
            } else if let Some(days) = days {
                query.push(("days", days.to_string()));
                if force {
                    query.push(("force", "true".into()));
                }
            }
            let response = client
                .post(url)
                .bearer_auth(access_token)
                .query(&query)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            decode(response, &base_url).await
        })
        .await
    }

    pub async fn admin_upsert_rate(
        &self,
        access_token: String,
        base_currency_code: Option<String>,
        target_currency_code: String,
        rate: f64,
        rate_date: Option<String>,
    ) -> Result<AdminManualRate, ApiError> {
        let url = format!("{}/admin/currencies/rates", self.base_url);
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        self.run(async move {
            let mut body = serde_json::json!({
                "target_currency_code": normalize_currency_code(&target_currency_code),
                "rate": rate,
            });
            if let Some(base) = base_currency_code.filter(|b| !b.trim().is_empty()) {
                body["base_currency_code"] = Value::String(normalize_currency_code(&base));
            }
            if let Some(date) = rate_date.filter(|d| !d.trim().is_empty()) {
                body["rate_date"] = Value::String(date);
            }
            let response = client
                .put(url)
                .bearer_auth(access_token)
                .json(&body)
                .send()
                .await
                .map_err(|error| ApiError::network(&base_url, error))?;
            let payload: UpsertRateResponse = decode(response, &base_url).await?;
            payload.rate.ok_or_else(|| ApiError {
                message: "The server saved the rate but returned no record.".into(),
                status: Some(500),
            })
        })
        .await
    }
}
