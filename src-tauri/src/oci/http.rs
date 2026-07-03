use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("network: {0}")]
    Network(String),
    #[error("timeout")]
    Timeout,
    #[error("status {status}: {body}")]
    Status { status: u16, body: String, retry_after_ms: Option<u64> },
    #[error("invalid url: {0}")]
    InvalidUrl(String),
}

impl HttpError {
    pub fn is_transient(&self) -> bool {
        match self {
            HttpError::Network(_) | HttpError::Timeout => true,
            HttpError::Status { status, .. } => *status >= 500 || *status == 429,
            HttpError::InvalidUrl(_) => false,
        }
    }
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            HttpError::Status { retry_after_ms, .. } => *retry_after_ms,
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    user_agent: String,
    max_retries: u32,
}

impl HttpClient {
    pub fn new() -> Self {
        let inner = reqwest::Client::builder()
            .user_agent(concat!("imagevue/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client");
        Self {
            inner,
            user_agent: concat!("imagevue/", env!("CARGO_PKG_VERSION")).to_string(),
            max_retries: 3,
        }
    }

    pub fn inner(&self) -> &reqwest::Client { &self.inner }
    pub fn user_agent(&self) -> &str { &self.user_agent }

    pub async fn execute(
        &self,
        mut req: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, HttpError> {
        req = req.header(reqwest::header::USER_AGENT, &self.user_agent);
        let mut attempt = 0u32;
        loop {
            let cloned = req
                .try_clone()
                .ok_or_else(|| HttpError::InvalidUrl("request not cloneable".into()))?;
            match cloned.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        || status.is_server_error()
                    {
                        let retry_after = resp
                            .headers()
                            .get(reqwest::header::RETRY_AFTER)
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse::<u64>().ok())
                            .map(|s| s * 1000);
                        if attempt < self.max_retries {
                            attempt += 1;
                            let delay = retry_after.unwrap_or(200u64 * (1 << attempt.min(5)));
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            continue;
                        }
                        let body = resp.text().await.unwrap_or_default();
                        return Err(HttpError::Status {
                            status: status.as_u16(),
                            body,
                            retry_after_ms: retry_after,
                        });
                    }
                    return Ok(resp);
                }
                Err(e) => {
                    if e.is_timeout() || e.is_connect() || e.is_request() {
                        if attempt < self.max_retries {
                            attempt += 1;
                            tokio::time::sleep(Duration::from_millis(200 * (1 << attempt.min(5)))).await;
                            continue;
                        }
                        return if e.is_timeout() { Err(HttpError::Timeout) } else { Err(HttpError::Network(e.to_string())) };
                    }
                    return Err(HttpError::Network(e.to_string()));
                }
            }
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn retries_on_500() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .mount(&server).await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server).await;
        let client = HttpClient::new();
        let url = format!("{}/", server.uri());
        let resp = client.execute(client.inner().get(&url)).await.unwrap();
        assert_eq!(resp.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn gives_up_after_max_retries() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server).await;
        let client = HttpClient::new();
        let url = format!("{}/", server.uri());
        let err = client.execute(client.inner().get(&url)).await.unwrap_err();
        assert!(matches!(err, HttpError::Status { status: 500, .. }));
    }

    #[test]
    fn is_transient_logic() {
        assert!(HttpError::Timeout.is_transient());
        assert!(HttpError::Network("x".into()).is_transient());
        assert!(HttpError::Status { status: 500, body: "".into(), retry_after_ms: None }.is_transient());
        assert!(HttpError::Status { status: 429, body: "".into(), retry_after_ms: None }.is_transient());
        assert!(!HttpError::Status { status: 404, body: "".into(), retry_after_ms: None }.is_transient());
    }
}
