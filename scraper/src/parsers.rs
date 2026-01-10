use serde_json::Value;
use crate::models::*;
use chrono::{DateTime, Utc, TimeZone};
use log::debug;
use ammonia;
use anyhow::{Result, Context};

// --- Parsing Trait ---

pub trait AtsParser {
    fn parse(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>>;
    fn estimate_raw_item_count(&self, data: &Value) -> usize;
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

pub(crate) fn clean_html(html: &str) -> String {
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
    fn parse(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        match self {
            AtsType::Greenhouse => self.parse_greenhouse(company, data),
            AtsType::Lever => self.parse_lever(company, data),
            AtsType::SmartRecruiters => self.parse_smartrecruiters(company, data),
            AtsType::Ashby => self.parse_ashby(company, data),
            AtsType::Workable => self.parse_workable(company, data),
            AtsType::Recruitee => self.parse_recruitee(company, data),
            AtsType::Breezy => self.parse_breezy(company, data),
            _ => Ok(vec![]),
        }
    }

    fn estimate_raw_item_count(&self, data: &Value) -> usize {
        match self {
            AtsType::Greenhouse => self.count_greenhouse(data),
            AtsType::Ashby => self.count_ashby(data),
            _ => 0,
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

    fn count_greenhouse(&self, data: &Value) -> usize {
        data["jobs"].as_array().map(|v| v.len())
            .or_else(|| if data.is_array() { data.as_array().map(|v| v.len()) } else { None })
            .unwrap_or(0)
    }

    fn count_ashby(&self, data: &Value) -> usize {
        data["jobs"].as_array().map(|v| v.len()).unwrap_or(0)
    }

    fn parse_greenhouse(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let raw_jobs = match self.get_raw_greenhouse_jobs(data) {
            Ok(jobs) => jobs,
            Err(e) => {
                let data_str = serde_json::to_string(data).unwrap_or_default();
                debug!("Failed Greenhouse JSON (first 500 chars): {:.500}", data_str);
                return Err(anyhow::anyhow!("Greenhouse parsing failed for {}: {}", company.name, e));
            }
        };

        Ok(raw_jobs.into_iter().map(|rj| {
            let is_edu_optional = self.is_greenhouse_education_optional(&rj);
            let mut job = self.new_job(company, rj.id.to_string(), rj.title, rj.url);
            
            job.description = rj.description.as_ref().map(|d| clean_html(d.as_str())).unwrap_or_default();
            job.posted = normalize_date(rj.posted.as_deref().unwrap_or_default());
            
            
            job.location = match &rj.location {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Object(map)) => {
                    map.get("name").and_then(|v| v.as_str()).map(String::from)
                        .or_else(|| map.get("city").and_then(|v| v.as_str()).map(String::from)) // Fallback to city
                        .unwrap_or_else(|| "Unknown".to_string())
                },
                _ => String::new(),
            };

            if is_edu_optional {
                job.tags.push("Education Optional".to_string());
            }

            job.departments = rj.departments.into_iter().filter_map(|d| d.name).collect();
            job.offices = rj.offices.into_iter().filter_map(|o| o.name).collect();

            job
        }).collect())
    }

    fn get_raw_greenhouse_jobs(&self, data: &Value) -> Result<Vec<RawGreenhouseJob>, serde_json::Error> {
        if let Some(jobs) = data.get("jobs").and_then(|v| v.as_array()) {
            serde_json::from_value::<Vec<RawGreenhouseJob>>(Value::Array(jobs.to_vec()))
        } else if let Ok(jobs) = serde_json::from_value::<Vec<RawGreenhouseJob>>(data.clone()) {
            Ok(jobs)
        } else {
            serde_json::from_value::<RawGreenhouseJob>(data.clone()).map(|j| vec![j])
        }
    }

    fn is_greenhouse_education_optional(&self, rj: &RawGreenhouseJob) -> bool {
        const EDU_OPTIONAL: &str = "education_optional";
        const EDU_FIELD: &str = "Education";
        
        let is_optional = |v: &str| v == EDU_OPTIONAL;

        rj.education.as_ref().map_or(false, |e| match e {
            GreenhouseEducation::Object { value } => is_optional(value),
            GreenhouseEducation::String(s) => is_optional(s),
        }) || rj.metadata.as_ref().map_or(false, |m| {
            m.iter().any(|item| {
                let name = item.name.as_deref().or(item.label.as_deref());
                if name == Some(EDU_FIELD) {
                    return item.value.as_str().map_or(false, is_optional) ||
                           item.value.get("value").and_then(|v| v.as_str()).map_or(false, is_optional);
                }
                false
            })
        })
    }

    fn parse_lever(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let items: Vec<LeverJob> = match serde_json::from_value(data.clone()) {
            Ok(j) => j,
            Err(e) => return Err(anyhow::anyhow!("Lever parsing failed for {}: {}", company.name, e)),
        };

        Ok(items.into_iter().map(|j| {
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
        }).collect())
    }

    fn parse_smartrecruiters(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let resp: SmartRecruitersResponse = serde_json::from_value(data.clone())
            .context(format!("SmartRecruiters parsing failed for {}", company.name))?;
        Ok(resp.content.into_iter().map(|j| {
            let url = j.posting_url.unwrap_or_else(|| format!("https://jobs.smartrecruiters.com/{}/{}", company.slug, j.id));
            let mut job = self.new_job(company, j.id.clone(), j.name, url);
            
            // Build location string
            let loc = &j.location;
            let mut loc_parts = Vec::new();
            if let Some(city) = &loc.city { if !city.is_empty() { loc_parts.push(city.as_str()); } }
            if let Some(region) = &loc.region { if !region.is_empty() { loc_parts.push(region.as_str()); } }
            if let Some(country) = &loc.country { if !country.is_empty() { loc_parts.push(country.as_str()); } }
            
            job.location = if loc_parts.is_empty() {
                loc.full_location.clone().unwrap_or_default()
            } else {
                loc_parts.join(", ")
            };
            
            job.posted = normalize_date(&j.released_date.unwrap_or_default());
            
            if let Some(dept) = j.department.and_then(|d| d.label) {
                if !dept.is_empty() { job.departments.push(dept); }
            }

            // Extract tags from custom fields or employment type
            if let Some(emp_type) = j.type_of_employment.and_then(|t| t.label) {
                if !emp_type.is_empty() { job.tags.push(emp_type); }
            }

            if let Some(custom_fields) = j.custom_field {
                for field in custom_fields {
                    // Example: "Remote", "Work Space", etc.
                    if field.field_label.contains("Work Space") || field.field_label.contains("Remote") {
                        if let Some(val) = field.value_label {
                            if !val.is_empty() { job.tags.push(val); }
                        }
                    }
                }
            }

            job
        }).collect())
    }

    fn parse_ashby(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let resp: AshbyResponse = match serde_json::from_value(data.clone()) {
            Ok(r) => r,
            Err(e) => return Err(anyhow::anyhow!("Ashby parsing failed for {}: {}", company.name, e)),
        };
        Ok(resp.jobs.into_iter().map(|j| {
            let mut job = self.new_job(company, j.id, j.title, j.job_url);
            job.location = match &j.location {
                 Some(Value::String(s)) => s.clone(),
                 Some(Value::Object(map)) => {
                    // Try common location fields
                    map.get("name").and_then(|v| v.as_str()).map(String::from)
                       .or_else(|| map.get("city").and_then(|v| v.as_str()).map(String::from))
                       .unwrap_or_default()
                 },
                 _ => String::new(),
            };
            job.posted = normalize_date(&j.published_at.unwrap_or_default());
            
            job.description = j.description_html.as_ref()
                .map(|d| clean_html(d.as_str()))
                .unwrap_or_default();

            if let Some(dept) = j.department {
                job.departments.push(dept);
            }
            job
        }).collect())
    }

    fn parse_workable(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let resp: WorkableResponse = serde_json::from_value(data.clone())
            .context(format!("Workable parsing failed for {}", company.name))?;
        Ok(resp.jobs.into_iter().map(|j| {
            let url = format!("https://apply.workable.com/{}/j/{}/", company.slug, j.shortcode);
            let mut job = self.new_job(company, j.shortcode.clone(), j.title, url);
            job.location = format!("{}, {}", j.city.unwrap_or_default(), j.country.unwrap_or_default());
            job.posted = normalize_date(&j.created_at.unwrap_or_default());
            
            // Build description from v2 API fields
            let mut desc = j.description.unwrap_or_default();
            if let Some(req) = j.requirements {
                if !req.is_empty() {
                    desc.push_str("<h3>Requirements</h3>");
                    desc.push_str(&req);
                }
            }
            if let Some(ben) = j.benefits {
                if !ben.is_empty() {
                    desc.push_str("<h3>Benefits</h3>");
                    desc.push_str(&ben);
                }
            }
            job.description = clean_html(&desc);
            
            job
        }).collect())
    }

    fn parse_recruitee(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let resp: RecruiteeResponse = serde_json::from_value(data.clone())
            .context(format!("Recruitee parsing failed for {}", company.name))?;
        Ok(resp.offers.into_iter().map(|j| {
            let mut job = self.new_job(company, j.id.to_string(), j.title, j.careers_url);
            job.description = clean_html(&j.description.unwrap_or_default());
            job.location = j.location.unwrap_or_default();
            job.posted = normalize_date(&j.created_at.unwrap_or_default());
            if let Some(dept) = j.department {
                job.departments.push(dept);
            }
            job
        }).collect())
    }

    fn parse_breezy(&self, company: &CompanyEntry, data: &Value) -> Result<Vec<Job>> {
        let items: Vec<BreezyJob> = serde_json::from_value(data.clone())
            .context(format!("Breezy parsing failed for {}", company.name))?;
        Ok(items.into_iter().map(|j| {
            let url = j.url.clone().unwrap_or_else(|| format!("https://{}.breezy.hr/p/{}", company.slug, j.id));
            let mut job = self.new_job(company, j.id, j.name, url);
            
            // Build location string
            if let Some(loc) = &j.location {
                let mut loc_parts = Vec::new();
                if let Some(name) = &loc.name { if !name.is_empty() { loc_parts.push(name.as_str()); } }
                if let Some(country) = &loc.country.as_ref().and_then(|c| c.name.as_ref()) {
                    if !country.is_empty() { loc_parts.push(country.as_str()); }
                }
                job.location = loc_parts.join(", ");

                // Tag remote
                if loc.is_remote == Some(true) {
                    job.tags.push("Remote".to_string());
                }
                if let Some(remote_label) = loc.remote_details.as_ref().and_then(|r| r.label.as_ref()) {
                    if !remote_label.is_empty() {
                        job.tags.push(remote_label.clone());
                    }
                }
            }

            job.posted = normalize_date(&j.published_date.unwrap_or_default());
            
            if let Some(dept) = j.department {
                if !dept.is_empty() { job.departments.push(dept); }
            }

            if let Some(emp_type) = j.employment_type.and_then(|t| t.name) {
                if !emp_type.is_empty() { job.tags.push(emp_type); }
            }

            if let Some(salary) = j.salary {
                if !salary.is_empty() { job.tags.push(format!("Salary: {}", salary)); }
            }

            job
        }).collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_smartrecruiters() {
        let company = CompanyEntry {
            name: "Air New Zealand".to_string(),
            ats_type: AtsType::SmartRecruiters,
            slug: "airnewzealand".to_string(),
            api_url: "https://api.smartrecruiters.com/v1/companies/airnewzealand/postings".to_string(),
            domain: Some("airnewzealand.com".to_string()),
        };

        let data = json!({
            "content": [
                {
                    "id": "6000000000788236",
                    "uuid": "9f599526-2f47-4d89-891b-d426a7715f00",
                    "name": "Senior Software Engineer (iOS)",
                     "company": { "name": "Air New Zealand", "identifier": "AirNewZealand" },
                    "releasedDate": "2026-01-08T21:57:15.644Z",
                    "location": {
                        "city": "Auckland",
                        "region": "Auckland",
                        "country": "nz",
                        "fullLocation": "Auckland, Auckland, New Zealand"
                    },
                    "typeOfEmployment": { "label": "Full-time" },
                    "customField": [
                        {
                            "fieldId": "6663765cd273aa35722c76da",
                            "fieldLabel": "Work Space ",
                            "valueLabel": "Auckland Airport - Campus (AKL35K)"
                        }
                    ]
                }
            ]
        });

        let jobs = AtsType::SmartRecruiters.parse(&company, &data).unwrap();
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(job.title, "Senior Software Engineer (iOS)");
        assert_eq!(job.location, "Auckland, Auckland, nz");
        assert_eq!(job.url, "https://jobs.smartrecruiters.com/airnewzealand/6000000000788236");
        assert!(job.tags.contains(&"Full-time".to_string()));
        assert!(job.tags.contains(&"Auckland Airport - Campus (AKL35K)".to_string()));
    }

    #[test]
    fn test_parse_breezy() {
        let company = CompanyEntry {
            name: "Cal.com".to_string(),
            ats_type: AtsType::Breezy,
            slug: "cal-com".to_string(),
            api_url: "https://cal-com.breezy.hr/json".to_string(),
            domain: Some("cal.com".to_string()),
        };

        let data = json!([
            {
                "id": "df04fa464882",
                "name": "Executive Assistant (EA)",
                "url": "https://cal-com.breezy.hr/p/df04fa464882-executive-assistant-ea",
                "published_date": "2026-01-09T13:27:24.490Z",
                "type": { "name": "Full-Time" },
                "location": {
                    "country": { "name": "United States" },
                    "is_remote": true,
                    "remote_details": { "label": "Fully remote, no location restrictions" },
                    "name": "United States"
                },
                "salary": "$60k"
            }
        ]);

        let jobs = AtsType::Breezy.parse(&company, &data).unwrap();
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(job.title, "Executive Assistant (EA)");
        assert_eq!(job.location, "United States, United States");
        assert_eq!(job.url, "https://cal-com.breezy.hr/p/df04fa464882-executive-assistant-ea");
        assert!(job.tags.contains(&"Full-Time".to_string()));
        assert!(job.tags.contains(&"Remote".to_string()));
        assert!(job.tags.contains(&"Fully remote, no location restrictions".to_string()));
        assert!(job.tags.contains(&"Salary: $60k".to_string()));
    }
}
