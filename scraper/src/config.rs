use std::env;

pub struct Config {
    pub slugs_file: String,
    pub concurrency: usize,
    pub keywords_regex: String,
    pub negative_keywords_regex: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            slugs_file: env::var("SLUGS_FILE").unwrap_or_else(|_| "slugs.json".to_string()),
            concurrency: env::var("CONCURRENCY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(25),
            keywords_regex: env::var("KEYWORDS_REGEX").unwrap_or_else(|_| r"(?i)\b(intern|apprentice|student|trainee|internship|fellowship|undergraduate|junior|jr|graduate|entry[-\s]level|associate)\b".to_string()),
            negative_keywords_regex: env::var("NEGATIVE_KEYWORDS_REGEX").unwrap_or_else(|_| r"(?i)\b(senior|snr|sr|principal|lead|staff|director|vp|head\s+of|manager)\b".to_string()),
        }
    }
}
