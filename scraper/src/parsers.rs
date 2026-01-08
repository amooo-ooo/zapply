use serde_json::Value;
use crate::models::*;
use chrono::{DateTime, Utc, TimeZone};
use ammonia;

// --- Parsing Trait ---

pub trait AtsParser {
    fn parse(&self, company: &CompanyEntry, data: &Value) -> Vec<Job>;
}

fn normalize_date(date_str: &str) -> String {
    if date_str.is_empty() { return String::new(); }
    
    // Try to parse as ISO 8601 (e.g., 2024-01-01T12:00:00Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return dt.with_timezone(&Utc).to_rfc3339();
    }

    // Try RFC 2822 (e.g., Mon, 02 Jan 2006 15:04:05 -0700)
    if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
        return dt.with_timezone(&Utc).to_rfc3339();
    }
    
    // Try to parse as Unix timestamp (seconds or milliseconds)
    if let Ok(ts) = date_str.parse::<i64>() {
        let dt = if ts > 10_000_000_000 {
            Utc.timestamp_millis_opt(ts).single()
        } else {
            Utc.timestamp_opt(ts, 0).single()
        };
        if let Some(dt) = dt {
            return dt.to_rfc3339();
        }
    }

    date_str.to_string()
}

fn clean_html(html: &str) -> String {
    if html.is_empty() { return String::new(); }
    
    // Decode common entities if it looks double-escaped
    let decoded = if html.contains("&lt;") || html.contains("&gt;") || html.contains("&amp;") {
        html.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
    } else {
        html.to_string()
    };

    ammonia::clean(&decoded)
}

impl AtsParser for AtsType {
    fn parse(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        match self {
            AtsType::Greenhouse => self.parse_greenhouse(company, data),
            AtsType::Lever => self.parse_lever(company, data),
            AtsType::SmartRecruiters => self.parse_smartrecruiters(company, data),
            AtsType::Ashby => self.parse_ashby(company, data),
            AtsType::Workable => self.parse_workable(company, data),
            AtsType::Recruitee => self.parse_recruitee(company, data),
            AtsType::Breezy => self.parse_breezy(company, data),
            _ => vec![],
        }
    }
}

impl AtsType {
    fn new_job(&self, company: &CompanyEntry, id: String, title: String, url: String) -> Job {
        let ats_str = serde_json::to_string(self).unwrap_or_default().trim_matches('"').to_lowercase();
        Job {
            id: format!("{}-{}", ats_str, id),
            title,
            description: String::new(),
            company: company.name.clone(),
            slug: company.slug.clone(),
            ats: *self,
            url,
            company_url: company.domain.clone(),
            location: String::new(),
            city: None,
            region: None,
            country: None,
            country_code: None,
            posted: String::new(),
            departments: vec![],
            offices: vec![],
            tags: vec![],
            degree_levels: vec![],
            subject_areas: vec![],
        }
    }

    fn parse_greenhouse(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let raw_jobs = match self.get_raw_greenhouse_jobs(data) {
            Ok(jobs) => jobs,
            Err(e) => {
                println!("[ERROR] Greenhouse parsing failed for {}: {}", company.name, e);
                return vec![];
            }
        };

        raw_jobs.into_iter().map(|rj| {
            let is_edu_optional = self.is_greenhouse_education_optional(&rj);
            let mut job = self.new_job(company, rj.id.to_string(), rj.title, rj.url);
            
            job.description = rj.description.as_ref().map(|d| clean_html(d.as_str())).unwrap_or_default();
            job.posted = normalize_date(rj.posted.as_deref().unwrap_or_default());
            
            job.location = match rj.location {
                Some(GreenhouseLocation::Object { name }) => name,
                Some(GreenhouseLocation::String(s)) => s,
                None => String::new(),
            };

            if is_edu_optional {
                job.tags.push("Education Optional".to_string());
            }

            job.departments = rj.departments.into_iter().filter_map(|d| d.name).collect();
            job.offices = rj.offices.into_iter().filter_map(|o| o.name).collect();

            job
        }).collect()
    }

    fn get_raw_greenhouse_jobs(&self, data: &Value) -> Result<Vec<RawGreenhouseJob>, serde_json::Error> {
        if let Some(jobs) = data.get("jobs").and_then(|v| v.as_array()) {
            serde_json::from_value::<Vec<RawGreenhouseJob>>(Value::Array(jobs.clone()))
        } else if let Ok(jobs) = serde_json::from_value::<Vec<RawGreenhouseJob>>(data.clone()) {
            Ok(jobs)
        } else {
            serde_json::from_value::<RawGreenhouseJob>(data.clone()).map(|j| vec![j])
        }
    }

    fn is_greenhouse_education_optional(&self, rj: &RawGreenhouseJob) -> bool {
        // Check top-level education field
        if let Some(edu) = &rj.education {
            let val = match edu {
                GreenhouseEducation::Object { value } => value,
                GreenhouseEducation::String(s) => s,
            };
            if val == "education_optional" { return true; }
        }

        // Check metadata
        if let Some(metadata) = &rj.metadata {
            for item in metadata {
                let name = item.name.as_ref().or(item.label.as_ref());
                if name.map(|s| s.as_str()) == Some("Education") {
                    let val_str = match &item.value {
                        Value::String(s) => Some(s.as_str()),
                        Value::Object(obj) => obj.get("value").and_then(|v| v.as_str()),
                        _ => None,
                    };
                    if val_str == Some("education_optional") {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn parse_lever(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let items: Vec<LeverJob> = serde_json::from_value(data.clone()).unwrap_or_default();

        items.into_iter().map(|j| {
            let mut job = self.new_job(company, j.id, j.text, j.hosted_url);
            job.description = clean_html(&j.description.unwrap_or_default());
            job.location = j.categories.location.unwrap_or_default();
            job.posted = normalize_date(&j.created_at.map(|c| c.to_string()).unwrap_or_default());
            
            let dept = j.categories.team.or(j.categories.department).unwrap_or_default();
            if !dept.is_empty() { job.departments.push(dept); }

            if let Some(commitment) = j.categories.commitment {
                if !commitment.is_empty() { job.tags.push(commitment); }
            }

            job
        }).collect()
    }

    fn parse_smartrecruiters(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let resp: SmartRecruitersResponse = serde_json::from_value(data.clone()).unwrap_or(SmartRecruitersResponse { content: vec![] });
        resp.content.into_iter().map(|j| {
            let url = format!("https://jobs.smartrecruiters.com/{}/{}", company.slug, j.id);
            let mut job = self.new_job(company, j.uuid, j.name, url);
            job.location = format!("{}, {}", j.location.city.unwrap_or_default(), j.location.country.unwrap_or_default());
            job.posted = normalize_date(&j.released_date.unwrap_or_default());
            if let Some(dept) = j.department.and_then(|d| d.label) {
                job.departments.push(dept);
            }
            job
        }).collect()
    }

    fn parse_ashby(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let resp: AshbyResponse = match serde_json::from_value(data.clone()) {
            Ok(r) => r,
            Err(e) => {
                println!("[ERROR] Ashby parsing failed for {}: {}", company.name, e);
                return vec![];
            }
        };
        resp.jobs.into_iter().map(|j| {
            let mut job = self.new_job(company, j.id, j.title, j.job_url);
            job.location = j.location.unwrap_or_default();
            job.posted = normalize_date(&j.published_at.unwrap_or_default());
            
            job.description = j.description_html.as_ref()
                .or(j.description_html.as_ref())
                .map(|d| clean_html(d.as_str()))
                .unwrap_or_default();

            if let Some(dept) = j.department {
                job.departments.push(dept);
            }
            job
        }).collect()
    }

    fn parse_workable(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let resp: WorkableResponse = serde_json::from_value(data.clone()).unwrap_or(WorkableResponse { jobs: vec![] });
        resp.jobs.into_iter().map(|j| {
            let url = format!("https://apply.workable.com/{}/j/{}/", company.slug, j.shortcode);
            let mut job = self.new_job(company, j.shortcode, j.title, url);
            job.location = format!("{}, {}", j.city.unwrap_or_default(), j.country.unwrap_or_default());
            job.posted = normalize_date(&j.created_at.unwrap_or_default());
            job
        }).collect()
    }

    fn parse_recruitee(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let resp: RecruiteeResponse = serde_json::from_value(data.clone()).unwrap_or(RecruiteeResponse { offers: vec![] });
        resp.offers.into_iter().map(|j| {
            let mut job = self.new_job(company, j.id.to_string(), j.title, j.careers_url);
            job.description = clean_html(&j.description.unwrap_or_default());
            job.location = j.location.unwrap_or_default();
            job.posted = normalize_date(&j.created_at.unwrap_or_default());
            if let Some(dept) = j.department {
                job.departments.push(dept);
            }
            job
        }).collect()
    }

    fn parse_breezy(&self, company: &CompanyEntry, data: &Value) -> Vec<Job> {
        let items: Vec<BreezyJob> = serde_json::from_value(data.clone()).unwrap_or_default();
        items.into_iter().map(|j| {
            let url = format!("https://{}.breezy.hr/p/{}", company.slug, j.id);
            let mut job = self.new_job(company, j.id, j.name, url);
            job.description = clean_html(&j.description.unwrap_or_default());
            job.location = j.location.and_then(|l| l.name).unwrap_or_default();
            job.posted = normalize_date(&j.updated_at.unwrap_or_default());
            if let Some(dept) = j.department {
                job.departments.push(dept);
            }
            job
        }).collect()
    }
}
