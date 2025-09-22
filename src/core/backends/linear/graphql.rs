use cynic::Operation;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONTENT_TYPE, RETRY_AFTER, USER_AGENT};
use reqwest::Client;
use std::time::Duration;
use url::Url;

use super::config::LinearConfig;

#[derive(thiserror::Error, Debug)]
pub enum LinearError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("rate limited; retry after {0:?}")]
    RateLimited(Option<Duration>),
    #[error("graphql errors: {0}")]
    GraphQl(String),
}

impl From<reqwest::Error> for LinearError {
    fn from(e: reqwest::Error) -> Self { Self::Transport(e.to_string()) }
}

pub fn build_client(cfg: &LinearConfig) -> Result<(Client, Url), LinearError> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", cfg.token).parse().map_err(|e| LinearError::Transport(e.to_string()))?);
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(USER_AGENT, cfg.user_agent.parse().unwrap());

    let client = Client::builder()
        .default_headers(headers)
        .timeout(cfg.timeout)
        .build()
        .map_err(LinearError::from)?;

    Ok((client, cfg.endpoint.clone()))
}


pub struct LinearGraphQl {
    client: Client,
    endpoint: Url,
    retry: super::config::RetryConfig,
}

impl LinearGraphQl {
    pub fn new(cfg: &LinearConfig) -> Result<Self, LinearError> {
        let (client, endpoint) = build_client(cfg)?;
        Ok(Self { client, endpoint, retry: cfg.retry.clone() })
    }

    pub async fn execute<O>(&self, op: O) -> Result<O::ResponseData, LinearError>
    where
        O: Operation + Clone,
    {
        self.run_with_retries(|| async {
            // Build request manually so we can inspect HTTP status/headers for retries.
            let body = cynic::http::Request::new(op.clone());
            let resp = self
                .client
                .post(self.endpoint.clone())
                .json(&body)
                .send()
                .await
                .map_err(LinearError::from)?;

            let status = resp.status();
            if status.as_u16() == 429 {
                // Extract Retry-After and signal to retry loop
                let wait = retry_after_to_duration(resp.headers().get(RETRY_AFTER));
                return Err(LinearError::RateLimited(wait));
            }

            if status.is_server_error() {
                return Err(LinearError::Transport(format!("server error: {}", status)));
            }

            if !status.is_success() {
                return Err(LinearError::Transport(format!("http error: {}", status)));
            }

            let gql_resp: cynic::http::GraphQlResponse<O::ResponseData> = resp
                .json()
                .await
                .map_err(|e| LinearError::Transport(e.to_string()))?;

            if let Some(errors) = gql_resp.errors {
                return Err(LinearError::GraphQl(format!("{:?}", errors)));
            }

            gql_resp
                .data
                .ok_or_else(|| LinearError::Transport("missing data".into()))
        })
        .await
    }

    async fn run_with_retries<F, Fut, T>(&self, mut op: F) -> Result<T, LinearError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, LinearError>>,
    {
        let mut attempt: usize = 0;
        let mut delay_ms = self.retry.initial_interval_ms as f64;
        loop {
            match op().await {
                Ok(val) => return Ok(val),
                Err(LinearError::RateLimited(wait)) => {
                    let sleep = wait.unwrap_or_else(|| Duration::from_millis(delay_ms as u64));
                    tokio::time::sleep(sleep).await;
                }
                Err(LinearError::Transport(e)) => {
                    // Retry on server-side or network-ish messages; simplistic classification
                    if attempt >= self.retry.max_retries { return Err(LinearError::Transport(e)); }
                    tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;
                }
                Err(other) => {
                    // GraphQL semantic errors are not retried
                    return Err(other);
                }
            }
            attempt += 1;
            // Exponential backoff with jitter
            delay_ms *= self.retry.multiplier;
            let jitter = (self.retry.jitter * delay_ms) as u64;
            if jitter > 0 {
                let jitter_part = (fastrand::u64(..=jitter)) as i64 - (jitter as i64 / 2);
                let new = (delay_ms as i64 + jitter_part).max(0) as u64;
                delay_ms = new as f64;
            }
        }
    }
}

fn retry_after_to_duration(header: Option<&reqwest::header::HeaderValue>) -> Option<Duration> {
    header.and_then(|hv| {
        if let Ok(s) = hv.to_str() {
            if let Ok(secs) = s.trim().parse::<u64>() { return Some(Duration::from_secs(secs)); }
            if let Ok(dt) = httpdate::parse_http_date(s) {
                let now = std::time::SystemTime::now();
                if let Ok(d) = dt.duration_since(now) { return Some(d); }
            }
        }
        None
    })
}

pub async fn execute_raw_for_tests(
    &self,
    query: &str,
    variables: &serde_json::Value,
) -> Result<serde_json::Value, LinearError> {
    self.run_with_retries(|| async {
        let body = serde_json::json!({ "query": query, "variables": variables });
        let resp = self
            .client
            .post(self.endpoint.clone())
            .json(&body)
            .send()
            .await
            .map_err(LinearError::from)?;

        let status = resp.status();
        if status.as_u16() == 429 {
            let wait = retry_after_to_duration(resp.headers().get(RETRY_AFTER));
            return Err(LinearError::RateLimited(wait));
        }
        if status.is_server_error() {
            return Err(LinearError::Transport(format!("server error: {}", status)));
        }
        if !status.is_success() {
            return Err(LinearError::Transport(format!("http error: {}", status)));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LinearError::Transport(e.to_string()))?;

        if let Some(errors) = json.get("errors") {
            return Err(LinearError::GraphQl(errors.to_string()));
        }
        Ok(json.get("data").cloned().unwrap_or(serde_json::Value::Null))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    // Minimal operation aligned with our tiny schema
    #[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query")]
    struct Health {
        pub health: Option<String>,
    }

    #[derive(cynic::QueryBuilder, Debug, Clone)]
    #[cynic(graphql_type = "Query")]
    struct HealthQuery {
        #[cynic(flatten)]
        health: Health,
    }

    #[tokio::test]
    async fn retries_on_429_then_succeeds() {
        let server = MockServer::start();
        // First 429 with Retry-After: 0
        let _m1 = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(429)
                .header("Retry-After", "0")
                .body("{}");
        });
        // Then success
        let _m2 = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("{\"data\":{\"health\":\"ok\"}}");
        });

        let cfg = LinearConfig {
            endpoint: Url::parse(&server.url("/")) .unwrap(),
            token: "test".into(),
            user_agent: "foundry-mcp-linear/test".into(),
            timeout: Duration::from_secs(5),
            retry: super::super::config::RetryConfig { max_retries: 3, initial_interval_ms: 1, multiplier: 2.0, jitter: 0.0 },
        };
        let client = LinearGraphQl::new(&cfg).unwrap();
        let _ = client.execute(HealthQuery {}).await.unwrap();
    }
}
