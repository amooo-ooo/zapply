mod models;
mod parsers;
mod tag;
mod location; 
mod config; 

use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;

use crate::models::{Job, CompanyEntry, AtsType, WorkableDetail, SmartRecruitersDetail, RecruiteeDetailResponse};
use crate::parsers::{AtsParser, clean_html};
use crate::tag::{TagEngine, EducationDetector};
use crate::location::LocationEngine;
use crate::config::Config;
use log::{info, warn, error, debug};

// --- Database Abstraction ---

#[derive(Serialize, Clone)]
pub struct DbQuery {
    pub sql: String,
    pub params: Vec<Value>,
}

// Static regex for parameter replacement (compiled once)
static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\?(\d+)").unwrap());

impl DbQuery {
    pub fn to_sql(&self) -> String {
        if self.params.is_empty() {
            return self.sql.clone();
        }

        // Create a map of index -> formatted value
        let formatted_params: HashMap<usize, String> = self.params.iter().enumerate().map(|(i, param)| {
             (i + 1, match param {
                Value::String(s) => format!("'{}'", escape_sql_string(s)),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() }, // SQLite uses 1/0 for bools
                Value::Null => "NULL".to_string(),
                _ => "NULL".to_string(), // Arrays/Objects shouldn't be passed directly usually
            })
        }).collect();

        // Use static regex
        PARAM_REGEX.replace_all(&self.sql, |caps: &regex::Captures| {
            if let Ok(idx) = caps[1].parse::<usize>() {
                 formatted_params.get(&idx).cloned().unwrap_or_else(|| caps[0].to_string())
            } else {
                caps[0].to_string()
            }
        }).to_string()
    }
}

fn escape_sql_string(input: &str) -> String {
    input.replace('\'', "''")
}

#[async_trait::async_trait]
trait JobDb: Send + Sync {
    async fn execute_batch(&self, queries: &[DbQuery]) -> Result<()>;
    async fn get_existing_ids(&self) -> Result<HashSet<String>>;
    async fn initialize_geo_tables(&self, countries: &HashMap<String, String>, regions: &HashMap<String, String>) -> Result<()>;
    async fn insert_jobs(&self, jobs: &[Job]) -> Result<()> {
        if jobs.is_empty() { return Ok(()); }
        
        let mut queries = Vec::new();
        
        // Batch DELETE for junction tables (one query per table for all jobs)
        if !jobs.is_empty() {
            let job_ids: Vec<Value> = jobs.iter().map(|j| Value::String(j.id.clone())).collect();
            let placeholders: String = (1..=job_ids.len()).map(|i| format!("?{}", i)).collect::<Vec<_>>().join(", ");
            
            queries.push(DbQuery {
                sql: format!("DELETE FROM job_degree_levels WHERE job_id IN ({})", placeholders),
                params: job_ids.clone(),
            });
            queries.push(DbQuery {
                sql: format!("DELETE FROM job_subject_areas WHERE job_id IN ({})", placeholders),
                params: job_ids.clone(),
            });
            queries.push(DbQuery {
                sql: format!("DELETE FROM job_departments WHERE job_id IN ({})", placeholders),
                params: job_ids.clone(),
            });
            queries.push(DbQuery {
                sql: format!("DELETE FROM job_offices WHERE job_id IN ({})", placeholders),
                params: job_ids.clone(),
            });
            queries.push(DbQuery {
                sql: format!("DELETE FROM job_tags WHERE job_id IN ({})", placeholders),
                params: job_ids.clone(),
            });
        }
        
        for job in jobs {
            // UPSERT main job record with change detection
            queries.push(DbQuery {
                sql: r#"INSERT INTO jobs (id, title, description, company, slug, ats,url, company_url, location, city, region, country, country_code, posted) 
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                        ON CONFLICT(id) DO UPDATE SET
                            title = excluded.title,
                            description = excluded.description,
                            company = excluded.company,
                            slug = excluded.slug,
                            ats = excluded.ats,
                            url = excluded.url,
                            company_url = excluded.company_url,
                            location = excluded.location,
                            city = excluded.city,
                            region = excluded.region,
                            country = excluded.country,
                            country_code = excluded.country_code,
                            posted = excluded.posted
                        WHERE 
                            jobs.title != excluded.title OR
                            jobs.description != excluded.description OR
                            jobs.location != excluded.location OR
                            jobs.city IS NOT excluded.city OR
                            jobs.region IS NOT excluded.region OR
                            jobs.country IS NOT excluded.country OR
                            jobs.country_code IS NOT excluded.country_code"#.to_string(),
                params: vec![
                    Value::String(job.id.clone()),
                    Value::String(job.title.clone()),
                    Value::String(job.description.clone()),
                    Value::String(job.company.clone()),
                    Value::String(job.slug.clone()),
                    Value::String(serde_json::to_string(&job.ats)?),
                    Value::String(job.url.clone()),
                    job.company_url.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
                    Value::String(job.location.clone()),
                    job.city.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
                    job.region.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
                    job.country.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
                    job.country_code.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
                    Value::String(job.posted.clone()),
                ],
            });

            // Insert fresh junction table records
            for degree in &job.degree_levels {
                queries.push(DbQuery {
                    sql: "INSERT OR IGNORE INTO job_degree_levels (job_id, name) VALUES (?1, ?2)".to_string(),
                    params: vec![Value::String(job.id.clone()), Value::String(degree.clone())],
                });
            }
            for subject in &job.subject_areas {
                queries.push(DbQuery {
                    sql: "INSERT OR IGNORE INTO job_subject_areas (job_id, name) VALUES (?1, ?2)".to_string(),
                    params: vec![Value::String(job.id.clone()), Value::String(subject.clone())],
                });
            }

            for dept in &job.departments {
                queries.push(DbQuery {
                    sql: "INSERT OR IGNORE INTO job_departments (job_id, name) VALUES (?1, ?2)".to_string(),
                    params: vec![Value::String(job.id.clone()), Value::String(dept.clone())],
                });
            }
            for office in &job.offices {
                queries.push(DbQuery {
                    sql: "INSERT OR IGNORE INTO job_offices (job_id, name) VALUES (?1, ?2)".to_string(),
                    params: vec![Value::String(job.id.clone()), Value::String(office.clone())],
                });
            }
            for tag in &job.tags {
                queries.push(DbQuery {
                    sql: "INSERT OR IGNORE INTO job_tags (job_id, name) VALUES (?1, ?2)".to_string(),
                    params: vec![Value::String(job.id.clone()), Value::String(tag.clone())],
                });
            }
        }
        self.execute_batch(&queries).await
    }
}


fn run_wrangler(args: Vec<&str>) -> Result<std::process::Output> {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.arg("/C").arg("npx");
        c
    } else {
        std::process::Command::new("npx")
    };
    
    let output = cmd.args(["wrangler", "d1", "execute"]).args(args).output()?;
    Ok(output)
}

struct LocalWranglerD1 {
    database_name: String,
}

#[async_trait::async_trait]
impl JobDb for LocalWranglerD1 {
    async fn execute_batch(&self, queries: &[DbQuery]) -> Result<()> {
        for chunk in queries.chunks(1000) {
            let mut sql = String::new();
            sql.push_str("BEGIN TRANSACTION;\n");
            for query in chunk {
                sql.push_str(&query.to_sql());
                sql.push_str(";\n");
            }
            sql.push_str("COMMIT;\n");

            let timestamp = Utc::now().timestamp_millis();
            let temp_file = format!("temp_batch_{}_{}.sql", chunk.len(), timestamp);
            std::fs::write(&temp_file, &sql)?;

            let output = run_wrangler(vec![&self.database_name, "--local", "--file", &temp_file])?;
            let _ = std::fs::remove_file(&temp_file);

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                error!("Wrangler D1 execution failed: {}", err);
                return Err(anyhow::anyhow!("Wrangler D1 execution failed: {}", err));
            }
        }
        Ok(())
    }

    async fn get_existing_ids(&self) -> Result<HashSet<String>> {
        let output = run_wrangler(vec![&self.database_name, "--local", "--command", "SELECT id FROM jobs", "--json"])?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Wrangler D1 query failed: {}", err));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_start = stdout.find('[').or(stdout.find('{')).unwrap_or(0);
        let data: Value = serde_json::from_str(&stdout[json_start..])?;
        
        let mut ids = HashSet::new();
        if let Some(results) = data[0]["results"].as_array() {
            for row in results {
                if let Some(id) = row["id"].as_str() {
                    ids.insert(id.to_string());
                }
            }
        }
        Ok(ids)
    }

    async fn initialize_geo_tables(&self, countries: &HashMap<String, String>, regions: &HashMap<String, String>) -> Result<()> {
        // Check if data already exists
        let output = run_wrangler(vec![&self.database_name, "--local", "--command", "SELECT count(*) as count FROM countries", "--json"])?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json_start = stdout.find('[').or(stdout.find('{')).unwrap_or(0);
            if let Ok(data) = serde_json::from_str::<Value>(&stdout[json_start..]) {
                if let Some(results) = data[0]["results"].as_array() {
                    if let Some(count) = results.first().and_then(|r| r["count"].as_i64()) {
                        if count > 0 {
                            info!("Geo tables already initialized ({} countries found). Skipping...", count);
                            return Ok(());
                        }
                    }
                }
            }
        }

        let mut queries = Vec::new();
        for (code, name) in countries {
            queries.push(DbQuery {
                sql: "INSERT OR IGNORE INTO countries (code, name) VALUES (?1, ?2)".to_string(),
                params: vec![Value::String(code.clone()), Value::String(name.clone())],
            });
        }
        for (id, name) in regions {
            let country_code = id.split('.').next().unwrap_or("").to_string();
            queries.push(DbQuery {
                sql: "INSERT OR IGNORE INTO regions (id, country_code, name) VALUES (?1, ?2, ?3)".to_string(),
                params: vec![Value::String(id.clone()), Value::String(country_code), Value::String(name.clone())],
            });
        }
        self.execute_batch(&queries).await
    }
}

struct RemoteD1 {
    client: reqwest::Client,
    account_id: String,
    database_id: String,
    api_token: String,
}

#[async_trait::async_trait]
impl JobDb for RemoteD1 {
    async fn execute_batch(&self, queries: &[DbQuery]) -> Result<()> {
        for chunk in queries.chunks(50) {
            let url = format!("https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/raw", self.account_id, self.database_id);
            
            // Combine all statements into a single SQL string with semicolons
            let combined_sql: String = chunk.iter()
                .map(|q| q.to_sql())
                .collect::<Vec<_>>()
                .join("; ");
            
            let payload = serde_json::json!({ "sql": combined_sql });
            let resp = self.client.post(&url)
                .bearer_auth(&self.api_token)
                .json(&payload)
                .send()
                .await?;

            if !resp.status().is_success() {
                let text = resp.text().await?;
                return Err(anyhow::anyhow!("D1 API Error: {}", text));
            }
        }
        Ok(())
    }

    async fn get_existing_ids(&self) -> Result<HashSet<String>> {
        let url = format!("https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query", self.account_id, self.database_id);
        let payload = DbQuery {
            sql: "SELECT id FROM jobs".to_string(),
            params: vec![],
        };

        let resp = self.client.post(&url)
            .bearer_auth(&self.api_token)
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(anyhow::anyhow!("D1 API Error: {}", text));
        }

        let data: Value = resp.json().await?;
        let mut ids = HashSet::new();
        if let Some(results) = data["result"][0]["results"].as_array() {
            for row in results {
                if let Some(id) = row["id"].as_str() {
                    ids.insert(id.to_string());
                }
            }
        }
        Ok(ids)
    }

    async fn initialize_geo_tables(&self, countries: &HashMap<String, String>, regions: &HashMap<String, String>) -> Result<()> {
        let mut queries = Vec::new();
        for (code, name) in countries {
            queries.push(DbQuery {
                sql: "INSERT OR IGNORE INTO countries (code, name) VALUES (?1, ?2)".to_string(),
                params: vec![Value::String(code.clone()), Value::String(name.clone())],
            });
        }
        for (id, name) in regions {
            let country_code = id.split('.').next().unwrap_or("").to_string();
            queries.push(DbQuery {
                sql: "INSERT OR IGNORE INTO regions (id, country_code, name) VALUES (?1, ?2, ?3)".to_string(),
                params: vec![Value::String(id.clone()), Value::String(country_code), Value::String(name.clone())],
            });
        }
        self.execute_batch(&queries).await
    }
}

// --- Utilities ---

fn load_json<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path))
}

fn save_json<T: Serialize>(path: &str, data: &T) -> Result<()> {
    let content = serde_json::to_string_pretty(data)
        .context("Failed to serialize data to JSON")?;
    fs::write(path, content)
        .with_context(|| format!("Failed to write to file: {}", path))?;
    Ok(())
}

// --- Scraper Implementation ---

async fn enrich_job(client: &reqwest::Client, mut j: Job, company_slug: &str) -> Result<Job> {
    if !j.description.is_empty() { return Ok(j); }

    match j.ats {
        AtsType::Workable => {
            let detail_url = format!("https://apply.workable.com/api/v2/accounts/{}/jobs/{}", company_slug, j.id.strip_prefix("workable-").unwrap_or(&j.id));
            if let Ok(resp) = client.get(&detail_url).send().await {
                if let Ok(detail) = resp.json::<WorkableDetail>().await {
                    let mut desc = detail.description.unwrap_or_default();
                    if let Some(req) = detail.requirements {
                        desc.push_str("<h3>Requirements</h3>");
                        desc.push_str(&req);
                    }
                    if let Some(ben) = detail.benefits {
                        desc.push_str("<h3>Benefits</h3>");
                        desc.push_str(&ben);
                    }
                    j.description = clean_html(&desc);
                }
            }
        }
        AtsType::SmartRecruiters => {
            let job_id = j.id.strip_prefix("smartrecruiters-").unwrap_or(&j.id);
            let detail_url = format!("https://api.smartrecruiters.com/v1/companies/{}/postings/{}", company_slug, job_id);
            
            if let Ok(resp) = client.get(&detail_url).send().await {
                if resp.status().is_success() {
                    if let Ok(detail) = resp.json::<SmartRecruitersDetail>().await {
                        let mut desc = String::new();
                        if let Some(sec) = detail.job_ad.sections.job_description {
                            if let Some(text) = sec.text { desc.push_str(&text); }
                        }
                        if let Some(sec) = detail.job_ad.sections.qualifications {
                            if let Some(text) = sec.text { 
                                desc.push_str("<h3>Qualifications</h3>");
                                desc.push_str(&text); 
                            }
                        }
                        if let Some(sec) = detail.job_ad.sections.additional_information {
                            if let Some(text) = sec.text { 
                                desc.push_str("<h3>Additional Information</h3>");
                                desc.push_str(&text); 
                            }
                        }
                        j.description = clean_html(&desc);
                    }
                }
            }
        }
        AtsType::Recruitee => {
            if let Some(slug) = j.url.split("/o/").last() {
                let detail_url = format!("https://{}.recruitee.com/api/offers/{}", company_slug, slug);
                if let Ok(resp) = client.get(&detail_url).send().await {
                    if let Ok(detail) = resp.json::<RecruiteeDetailResponse>().await {
                        let mut desc = detail.offer.description.unwrap_or_default();
                        if let Some(req) = detail.offer.requirements {
                            desc.push_str("<h3>Requirements</h3>");
                            desc.push_str(&req);
                        }
                        if let Some(ben) = detail.offer.benefits {
                            desc.push_str("<h3>Benefits</h3>");
                            desc.push_str(&ben);
                        }
                        j.description = clean_html(&desc);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(j)
}

fn normalize_job(
    mut j: Job, 
    company: &CompanyEntry, 
    tag_engine: &TagEngine, 
    edu_detector: &EducationDetector, 
    location_engine: &LocationEngine
) -> Job {
    j.company_url = company.domain.clone();

    // 1. Detect tags
    let mut unique_tags = HashSet::new();
    unique_tags.extend(j.tags);
    unique_tags.extend(tag_engine.detect_tags(&j.description).into_iter().map(String::from));
    unique_tags.extend(tag_engine.detect_tags(&j.title).into_iter().map(String::from));
    j.tags = unique_tags.into_iter().collect();
    
    // 2. Detect education info
    let combined_text = format!("{} {}", j.title, j.description);
    let edu_info = edu_detector.detect(&combined_text);
    j.degree_levels = edu_info.degree_levels;
    j.subject_areas = edu_info.subject_areas;
    
    // 3. Normalize location
    let loc_info = location_engine.resolve(&j.location);
    let formatted = loc_info.display_format();
    if !formatted.is_empty() {
        j.location = formatted;
    }
    j.city = loc_info.city;
    j.region = loc_info.region;
    j.country = loc_info.country;
    j.country_code = loc_info.country_code;
    
    if loc_info.work_mode != crate::models::WorkMode::InOffice {
        let mode_str = match loc_info.work_mode {
            crate::models::WorkMode::Remote => "Remote",
            crate::models::WorkMode::Hybrid => "Hybrid",
            _ => "",
        };
        if !mode_str.is_empty() {
            j.tags.push(mode_str.to_string());
        }
    }
    j
}

async fn process_company(
    client: &reqwest::Client,
    company: &CompanyEntry,
    keyword_regex: &Regex,
    negative_regex: &Regex,
    tag_engine: Arc<TagEngine>,
    edu_detector: Arc<EducationDetector>,
    location_engine: Arc<LocationEngine>
) -> Result<Vec<Job>> {
    let mut url = company.api_url.clone();
    if company.ats_type == AtsType::Greenhouse && !url.contains("content=true") {
        url.push_str(if url.contains('?') { "&content=true" } else { "?content=true" });
    }
    
    // Debug log for target ATS types
    if matches!(company.ats_type, AtsType::Greenhouse | AtsType::Ashby) {
        info!("Processing {:?} for {}: URL={}", company.ats_type, company.name, url);
    }

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        warn!("HTTP {} for {} ({})", resp.status(), url, company.name);
        return Err(anyhow::anyhow!("HTTP {} for {}", resp.status(), url));
    }
    
    let body_text = resp.text().await?;
    if matches!(company.ats_type, AtsType::Greenhouse | AtsType::Ashby) {
        debug!("Response for {}: {:.100}...", company.name, body_text);
    }

    let data: Value = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("JSON decode error for {}: {}", url, e))?;

    let jobs = company.ats_type.parse(company, &data)?;
    
    // --- Observability Check ---
    if matches!(company.ats_type, AtsType::Greenhouse | AtsType::Ashby) {
        let raw_item_count = company.ats_type.estimate_raw_item_count(&data);

        if raw_item_count > 0 && jobs.is_empty() {
            warn!("PARSING HEALTH ALERT: {} returned {} raw items but parsed 0 jobs. Check schema!", company.name, raw_item_count);
        } else {
             info!("Parsed {} jobs (from ~{} raw items) for {}", jobs.len(), raw_item_count, company.name);
        }
    } else {
        debug!("Parsed {} jobs for {}", jobs.len(), company.name);
    }
    // ---------------------------

    
    let now = Utc::now();
    let cutoff_default = now - Duration::days(60); 
    let cutoff_eoi = now - Duration::days(120); 

    let enrichment_stream = stream::iter(jobs)
        .filter_map(|j| async move {
            let is_target = matches!(j.ats, AtsType::Greenhouse | AtsType::Ashby);
            
            if !keyword_regex.is_match(&j.title) { 
                if is_target { debug!("Dropping {} job '{}': No keyword match", j.company, j.title); }
                return None; 
            }
            if negative_regex.is_match(&j.title) { 
                if is_target { debug!("Dropping {} job '{}': Negative keyword match", j.company, j.title); }
                return None; 
            }
            
            let is_eoi = j.title.to_lowercase().contains("expression of interest") || j.title.to_lowercase().contains("eoi");
            let cutoff = if is_eoi { cutoff_eoi } else { cutoff_default };
            
            if !j.posted.is_empty() {
                if let Ok(p) = DateTime::parse_from_rfc3339(&j.posted) {
                    if p.with_timezone(&Utc) <= cutoff { 
                        if is_target { debug!("Dropping {} job '{}': Too old ({})", j.company, j.title, j.posted); }
                        return None; 
                    }
                }
            }
            Some(j)
        })
        .map(|j| {
            let client = client.clone();
            let slug = company.slug.clone();
            let company = company.clone();
            let tag_engine = tag_engine.clone();
            let edu_detector = edu_detector.clone();
            let location_engine = location_engine.clone();

            async move {
                match enrich_job(&client, j, &slug).await {
                    Ok(enriched) => {
                         let normalized = normalize_job(enriched, &company, &tag_engine, &edu_detector, &location_engine);
                         Some(normalized)
                    },
                    Err(_) => None
                }
            }
        })
        .buffer_unordered(10);

    let filtered_jobs: Vec<Job> = enrichment_stream
        .filter_map(|res| async { res })
        .collect().await;

    Ok(filtered_jobs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("Normal String"), "Normal String");
        assert_eq!(escape_sql_string("O'Reilly"), "O''Reilly");
        assert_eq!(escape_sql_string("Multiple ' ' quotes"), "Multiple '' '' quotes");
        assert_eq!(escape_sql_string(""), "");
    }

    #[test]
    fn test_db_query_to_sql() {
        let query = DbQuery {
            sql: "INSERT INTO table (col1, col2, col3) VALUES (?1, ?2, ?3)".to_string(),
            params: vec![
                Value::String("O'Reilly".to_string()),
                Value::Number(serde_json::Number::from(42)),
                Value::Bool(true),
            ],
        };
        let sql = query.to_sql();
        assert_eq!(sql, "INSERT INTO table (col1, col2, col3) VALUES ('O''Reilly', 42, 1)");
    }
    
    #[test]
    fn test_db_query_to_sql_order() {
         let query = DbQuery {
            sql: "SELECT * FROM t WHERE id = ?2 AND name = ?1".to_string(),
            params: vec![
                Value::String("Test".to_string()),
                Value::Number(serde_json::Number::from(100)),
            ],
        };
        let sql = query.to_sql();
        assert_eq!(sql, "SELECT * FROM t WHERE id = 100 AND name = 'Test'");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().collect();
    let is_verbose = args.iter().any(|a| a == "--log");
    let default_level = if is_verbose { "info" } else { "error" };

    env_logger::init_from_env(env_logger::Env::default().default_filter_or(default_level));
    
    if is_verbose {
        info!("Starting Zapply Job Scraper (Rust)...");
    }
    let is_prod = args.iter().any(|a| a == "--prod");

    let db: Box<dyn JobDb> = if is_prod {
        info!("Mode: PROD (Remote D1)");
        Box::new(RemoteD1 {
            client: reqwest::Client::new(),
            account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID").context("CLOUDFLARE_ACCOUNT_ID not set")?,
            database_id: std::env::var("CLOUDFLARE_DATABASE_ID").context("CLOUDFLARE_DATABASE_ID not set")?,
            api_token: std::env::var("CLOUDFLARE_API_TOKEN").context("CLOUDFLARE_API_TOKEN not set")?,
        })
    } else {
        info!("Mode: DEV (Local Wrangler D1)");
        Box::new(LocalWranglerD1 {
            database_name: "zapply".to_string(),
        })
    };

    
    let config = Config::load();
    let keyword_regex = Regex::new(&config.keywords_regex).context("Invalid Regex")?;
    let negative_regex = Regex::new(&config.negative_keywords_regex).context("Invalid Negative Regex")?;

    info!("Loading company list...");
    let mut companies: Vec<CompanyEntry> = load_json(&config.slugs_file)
        .context(format!("Failed to load {}", config.slugs_file))?;

    if let Some(limit) = args.iter().find_map(|a| a.strip_prefix("--limit=")).and_then(|s| s.parse().ok()) {
        info!("Limiting search to {} companies.", limit);
        companies.truncate(limit);
    }

    info!("Fetching existing job IDs from database...");
    let existing_ids = db.get_existing_ids().await?;
    let mut cache: HashSet<String> = load_json(&config.cache_file).unwrap_or_default();
    cache.extend(existing_ids);
    
    let log_file = args.iter()
        .find_map(|a| a.strip_prefix("--log-file="))
        .and_then(|path| fs::File::create(path).ok())
        .map(|f| Arc::new(Mutex::new(f)));

    let mut location_engine = LocationEngine::new();
    if let Err(e) = location_engine.load_geonames("cities15000.txt", "admin1CodesASCII.txt", "countryInfo.txt") {
        warn!("Failed to load location data: {}. Location normalization will be limited.", e);
    } else {
        info!("Initializing geo tables in database...");
        db.initialize_geo_tables(&location_engine.countries, &location_engine.regions).await?;
    }

    let tag_engine = Arc::new(TagEngine::new());
    let edu_detector = Arc::new(EducationDetector::new());
    let location_engine = Arc::new(location_engine);
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let total = companies.len();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#> -"));

    let jobs_count = Arc::new(AtomicUsize::new(0));
    let failures_count = Arc::new(AtomicUsize::new(0));
    let inserted_count = Arc::new(AtomicUsize::new(0));

    const BATCH_SIZE: usize = 100;
    let batch_buffer = Arc::new(Mutex::new(Vec::new()));
    let cache = Arc::new(Mutex::new(cache));
    let db = Arc::new(db);

    let mut stream = stream::iter(companies)
        .map(|company| {
            let client = client.clone();
            let keyword_regex = keyword_regex.clone();
            let negative_regex = negative_regex.clone();
            let tag_engine = tag_engine.clone();
            let edu_detector = edu_detector.clone();
            let location_engine = location_engine.clone();
            let log_file = log_file.clone();
            let pb = pb.clone();
            let jobs_count = jobs_count.clone();
            let failures_count = failures_count.clone();
            let inserted_count = inserted_count.clone();
            let batch_buffer = batch_buffer.clone();
            let cache = cache.clone();
            let db = db.clone();

            async move {
                let result = process_company(&client, &company, &keyword_regex, &negative_regex, tag_engine, edu_detector, location_engine).await;
                let jobs = match result {
                    Ok(j) => {
                        jobs_count.fetch_add(j.len(), Ordering::SeqCst);
                        if let Some(ref f) = log_file {
                            let mut f = f.lock().unwrap();
                            writeln!(f, "[SUCCESS] {}: Found {} roles", company.name, j.len()).ok();
                        }
                        j
                    }
                    Err(e) => {
                        failures_count.fetch_add(1, Ordering::SeqCst);
                        if let Some(ref f) = log_file {
                            let mut f = f.lock().unwrap();
                            writeln!(f, "[ERROR] {}: {:#}", company.name, e).ok();
                        }
                        vec![]
                    }
                };

                // Add to batch buffer
                let mut buffer = batch_buffer.lock().unwrap();
                let mut cache_guard = cache.lock().unwrap();
                
                for job in jobs {
                    if cache_guard.insert(job.id.clone()) {
                        buffer.push(job);
                    }
                }

                // Check if we need to flush
                let should_flush = buffer.len() >= BATCH_SIZE;
                let jobs_to_insert = if should_flush {
                    std::mem::take(&mut *buffer)
                } else {
                    Vec::new()
                };
                drop(buffer);
                drop(cache_guard);

                // Flush batch if needed
                if !jobs_to_insert.is_empty() {
                    if let Err(e) = db.insert_jobs(&jobs_to_insert).await {
                        warn!("Failed to insert batch: {}", e);
                    } else {
                        let count = jobs_to_insert.len();
                        inserted_count.fetch_add(count, Ordering::SeqCst);
                    }
                }

                pb.inc(1);
                pb.set_message(format!("Jobs: {} | Inserted: {} | Failures: {}", 
                    jobs_count.load(Ordering::SeqCst),
                    inserted_count.load(Ordering::SeqCst),
                    failures_count.load(Ordering::SeqCst)
                ));
            }
        })
        .buffer_unordered(config.concurrency);

    // Process all companies
    while stream.next().await.is_some() {}

    // Flush remaining jobs
    let remaining_jobs = {
        let mut buffer = batch_buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    };

    if !remaining_jobs.is_empty() {
        db.insert_jobs(&remaining_jobs).await?;
        inserted_count.fetch_add(remaining_jobs.len(), Ordering::SeqCst);
    }

    pb.finish_with_message(format!("Done! Inserted {} jobs.", inserted_count.load(Ordering::SeqCst)));

    // Save updated cache
    let final_cache = {
        let cache_guard = cache.lock().unwrap();
        cache_guard.iter().cloned().collect::<Vec<_>>()
    };
    save_json(&config.cache_file, &final_cache)?;

    Ok(())
}
