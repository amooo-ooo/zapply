mod models;
mod parsers;
mod tag;
mod location; 

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

use crate::models::{Job, CompanyEntry, AtsType};
use crate::parsers::AtsParser;
use crate::tag::{TagEngine, EducationDetector};
use crate::location::LocationEngine;

// --- Configuration & Types ---

// --- Configuration & Types ---

pub struct Config {
    pub slugs_file: &'static str,
    pub cache_file: &'static str,
    pub concurrency: usize,
    pub keywords_regex: &'static str,
}

const CONFIG: Config = Config {
    slugs_file: "slugs.json",
    cache_file: "cache.json",
    concurrency: 50,
    keywords_regex: r"(?i)\b(intern|apprentice|student|trainee|internship|fellowship|undergraduate)\b",
};

// --- Database Abstraction ---

#[derive(Serialize, Clone)]
pub struct DbQuery {
    pub sql: String,
    pub params: Vec<Value>,
}

#[async_trait::async_trait]
trait JobDb: Send + Sync {
    async fn execute_batch(&self, queries: &[DbQuery]) -> Result<()>;
    async fn get_existing_ids(&self) -> Result<HashSet<String>>;
    async fn initialize_geo_tables(&self, countries: &HashMap<String, String>, regions: &HashMap<String, String>) -> Result<()>;
    async fn insert_jobs(&self, jobs: &[Job]) -> Result<()> {
        if jobs.is_empty() { return Ok(()); }
        
        let mut queries = Vec::new();
        for job in jobs {
            queries.push(DbQuery {
                sql: "INSERT OR IGNORE INTO jobs (id, title, description, company, slug, ats, url, company_url, location, city, region, country, country_code, posted) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)".to_string(),
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
                let mut statement = query.sql.clone();
                for (i, param) in query.params.iter().enumerate().rev() {
                    let placeholder = format!("?{}", i + 1);
                    let val_str = match param {
                        Value::String(s) => format!("'{}'", s.replace("'", "''")),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => "NULL".to_string(),
                    };
                    statement = statement.replace(&placeholder, &val_str);
                }
                sql.push_str(&statement);
                sql.push_str(";\n");
            }
            sql.push_str("COMMIT;\n");

            let temp_file = format!("temp_batch_{}.sql", chunk.len());
            std::fs::write(&temp_file, &sql)?;

            let output = run_wrangler(vec![&self.database_name, "--local", "--file", &temp_file])?;
            let _ = std::fs::remove_file(&temp_file);

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
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
                            println!("[INFO] Geo tables already initialized ({} countries found). Skipping...", count);
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
                .map(|q| {
                    let mut statement = q.sql.clone();
                    for (i, param) in q.params.iter().enumerate().rev() {
                        let placeholder = format!("?{}", i + 1);
                        let val_str = match param {
                            Value::String(s) => format!("'{}'", s.replace("'", "''")),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => "NULL".to_string(),
                        };
                        statement = statement.replace(&placeholder, &val_str);
                    }
                    statement
                })
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

struct Scraper {
    client: reqwest::Client,
    keyword_regex: Regex,
    cache: HashSet<String>,
    log_file: Option<Arc<Mutex<fs::File>>>,
    tag_engine: Arc<TagEngine>,
    edu_detector: Arc<EducationDetector>,
    location_engine: Arc<LocationEngine>,
}

impl Scraper {
    fn new(keyword_regex: Regex, cache: HashSet<String>, log_file: Option<fs::File>, location_engine: LocationEngine) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("Zapply/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let log_file = log_file.map(|f| Arc::new(Mutex::new(f)));
        let tag_engine = Arc::new(TagEngine::new());
        let edu_detector = Arc::new(EducationDetector::new());
        let location_engine = Arc::new(location_engine);
        Ok(Self { client, keyword_regex, cache, log_file, tag_engine, edu_detector, location_engine })
    }

    async fn process_company(client: &reqwest::Client, company: &CompanyEntry, regex: &Regex, tag_engine: &TagEngine, edu_detector: &EducationDetector, location_engine: &LocationEngine) -> Result<Vec<Job>> {
        let mut url = company.api_url.clone();
        if company.ats_type == AtsType::Greenhouse && !url.contains("content=true") {
            url.push_str(if url.contains('?') { "&content=true" } else { "?content=true" });
        }
        
        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {} for {}", resp.status(), url));
        }
        
        let data: Value = resp.json().await.map_err(|e| anyhow::anyhow!("JSON decode error for {}: {}", url, e))?;
        let jobs = company.ats_type.parse(company, &data);
        
        let now = Utc::now();
        let cutoff_default = now - Duration::days(30);
        let cutoff_eoi = now - Duration::days(90);

        let filtered: Vec<Job> = jobs.into_iter()
            .filter(|j| regex.is_match(&j.title))
            .filter(|j| {
                if j.posted.is_empty() { return true; }
                
                let title_lower = j.title.to_lowercase();
                let is_eoi = title_lower.contains("expression of interest") || title_lower.contains("eoi");
                let cutoff = if is_eoi { cutoff_eoi } else { cutoff_default };

                if let Ok(parsed) = DateTime::parse_from_rfc3339(&j.posted) {
                    return parsed.with_timezone(&Utc) > cutoff;
                }
                true
            })
            .map(|mut j| {
                // Populate company domain
                j.company_url = company.domain.clone();

                // Populate company domain
                j.company_url = company.domain.clone();

                // Detect tags
                let mut unique_tags = HashSet::new();
                unique_tags.extend(j.tags);
                unique_tags.extend(tag_engine.detect_tags(&j.description).into_iter().map(String::from));
                unique_tags.extend(tag_engine.detect_tags(&j.title).into_iter().map(String::from));
                j.tags = unique_tags.into_iter().collect();
                
                // Detect education info
                let combined_text = format!("{} {}", j.title, j.description);
                let edu_info = edu_detector.detect(&combined_text);
                j.degree_levels = edu_info.degree_levels;
                j.subject_areas = edu_info.subject_areas;
                
                // Normalize location
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
            })
            .collect();
        Ok(filtered)
    }

    async fn run(&mut self, companies: Vec<CompanyEntry>) -> Vec<Job> {
        let total = companies.len();
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"));

        let client = Arc::new(self.client.clone());
        let regex = Arc::new(self.keyword_regex.clone());
        let success_count = Arc::new(AtomicUsize::new(0));
        let fail_count = Arc::new(AtomicUsize::new(0));
        let jobs_count = Arc::new(AtomicUsize::new(0));
        let log_file = self.log_file.clone();
        let tag_engine = self.tag_engine.clone();
        let edu_detector = self.edu_detector.clone();
        let location_engine = self.location_engine.clone();

        let results = stream::iter(companies)
            .map(|company| {
                let client = client.clone();
                let regex = regex.clone();
                let success_count = success_count.clone();
                let fail_count = fail_count.clone();
                let jobs_count = jobs_count.clone();
                let log_file = log_file.clone();
                let pb = pb.clone();
                let tag_engine = tag_engine.clone();
                let edu_detector = edu_detector.clone();
                let location_engine = location_engine.clone();

                async move {
                    let result = Self::process_company(&client, &company, &regex, &tag_engine, &edu_detector, &location_engine).await;
                    let jobs = match result {
                        Ok(j) => {
                            success_count.fetch_add(1, Ordering::SeqCst);
                            jobs_count.fetch_add(j.len(), Ordering::SeqCst);
                            if let Some(ref f) = log_file {
                                let mut f = f.lock().unwrap();
                                writeln!(f, "[SUCCESS] {}: Found {} roles", company.name, j.len()).ok();
                            }
                            j
                        }
                        Err(e) => {
                            fail_count.fetch_add(1, Ordering::SeqCst);
                            if let Some(ref f) = log_file {
                                let mut f = f.lock().unwrap();
                                writeln!(f, "[ERROR] {}: {}", company.name, e).ok();
                            }
                            vec![]
                        }
                    };
                    pb.inc(1);
                    pb.set_message(format!("Jobs: {}", jobs_count.load(Ordering::SeqCst)));
                    jobs
                }
            })
            .buffer_unordered(CONFIG.concurrency)
            .collect::<Vec<Vec<Job>>>()
            .await;

        pb.finish_with_message(format!("Done! Found {} total jobs.", jobs_count.load(Ordering::SeqCst)));

        let mut new_jobs = Vec::new();
        for company_jobs in results {
            for job in company_jobs {
                if self.cache.insert(job.id.clone()) {
                    new_jobs.push(job);
                }
            }
        }
        new_jobs
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    println!("[INFO] Starting Zapply Job Scraper (Rust)...");

    let args: Vec<String> = std::env::args().collect();
    let is_prod = args.iter().any(|a| a == "--prod");

    let db: Box<dyn JobDb> = if is_prod {
        println!("[INFO] Mode: PROD (Remote D1)");
        Box::new(RemoteD1 {
            client: reqwest::Client::new(),
            account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID").context("CLOUDFLARE_ACCOUNT_ID not set")?,
            database_id: std::env::var("CLOUDFLARE_DATABASE_ID").context("CLOUDFLARE_DATABASE_ID not set")?,
            api_token: std::env::var("CLOUDFLARE_API_TOKEN").context("CLOUDFLARE_API_TOKEN not set")?,
        })
    } else {
        println!("[INFO] Mode: DEV (Local Wrangler D1)");
        Box::new(LocalWranglerD1 {
            database_name: "zapply".to_string(),
        })
    };

    let keyword_regex = Regex::new(CONFIG.keywords_regex).context("Invalid Regex")?;
    println!("[INFO] Loading company list...");
    let mut companies: Vec<CompanyEntry> = load_json(CONFIG.slugs_file)
        .context(format!("Failed to load {}", CONFIG.slugs_file))?;

    if let Some(limit) = args.iter().find_map(|a| a.strip_prefix("--limit=")).and_then(|s| s.parse().ok()) {
        println!("[INFO] Limiting search to {} companies.", limit);
        companies.truncate(limit);
    }

    println!("[INFO] Fetching existing job IDs from database...");
    let existing_ids = db.get_existing_ids().await?;
    let mut cache: HashSet<String> = load_json(CONFIG.cache_file).unwrap_or_default();
    cache.extend(existing_ids);
    
    let log_file = args.iter()
        .find_map(|a| a.strip_prefix("--log-file="))
        .and_then(|path| fs::File::create(path).ok());

    let mut location_engine = LocationEngine::new();
    if let Err(e) = location_engine.load_geonames("cities15000.txt", "admin1CodesASCII.txt", "countryInfo.txt") {
        println!("[WARN] Failed to load location data: {}. Location normalization will be limited.", e);
    } else {
        println!("[INFO] Initializing geo tables in database...");
        db.initialize_geo_tables(&location_engine.countries, &location_engine.regions).await?;
    }

    let mut scraper = Scraper::new(keyword_regex, cache, log_file, location_engine)?;
    let new_jobs = scraper.run(companies).await;

    println!("[DONE] Found {} new early-career roles.", new_jobs.len());

    if !new_jobs.is_empty() {
        db.insert_jobs(&new_jobs).await?;
        println!("[INFO] Inserted {} new jobs into the database.", new_jobs.len());
    }

    save_json(CONFIG.cache_file, &scraper.cache.iter().cloned().collect::<Vec<_>>())?;

    Ok(())
}
