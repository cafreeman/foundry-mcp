use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_interval_ms: u64,
    pub multiplier: f64,
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_interval_ms: 250,
            multiplier: 2.0,
            jitter: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearConfig {
    pub endpoint: Url,
    pub token: String,
    pub user_agent: String,
    pub timeout: Duration,
    pub retry: RetryConfig,
    // Optional team resolution hints
    pub team_id: Option<String>,
    pub team_key: Option<String>,
    pub team_name: Option<String>,
}

impl LinearConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let endpoint = std::env::var("LINEAR_GRAPHQL_ENDPOINT")
            .unwrap_or_else(|_| "https://api.linear.app/graphql".to_string());
        let endpoint = Url::parse(&endpoint)?;

        let token = std::env::var("LINEAR_API_TOKEN")
            .or_else(|_| std::env::var("LINEAR_API_KEY"))
            .map_err(|_| {
                anyhow::anyhow!("Missing LINEAR_API_TOKEN or LINEAR_API_KEY in environment")
            })?;

        let user_agent = format!(
            "foundry-mcp-linear/{} (+https://github.com/cafreeman/foundry-mcp)",
            env!("CARGO_PKG_VERSION")
        );

        let timeout = Duration::from_secs(
            std::env::var("LINEAR_HTTP_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(30),
        );

        let team_id = std::env::var("LINEAR_TEAM_ID").ok();
        let team_key = std::env::var("LINEAR_TEAM_KEY").ok();
        let team_name = std::env::var("LINEAR_TEAM_NAME").ok();

        Ok(Self {
            endpoint,
            token,
            user_agent,
            timeout,
            retry: RetryConfig::default(),
            team_id,
            team_key,
            team_name,
        })
    }
}
