use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::WorkMode;
use std::fs::File;
use std::io::{BufRead, BufReader};
use anyhow::Result;
use log::info;

const REMOTE_KEYWORDS: &[&str] = &["remote", "anywhere", "wfh"];
const HYBRID_KEYWORDS: &[&str] = &["hybrid"];


use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub work_mode: WorkMode,
}

impl LocationInfo {
    pub fn display_format(&self) -> String {
        let mut parts = Vec::with_capacity(3);
        
        if let Some(city) = &self.city {
            parts.push(city.as_str());
        }
        
        if let Some(region) = &self.region {
            // Avoid "Singapore, Singapore" or "New York, New York" redundancy
            if self.city.as_deref() != Some(region) {
                parts.push(region.as_str());
            }
        }
        
        if let Some(country) = &self.country {
            // Avoid "Singapore, Singapore" if already covered
            if !parts.contains(&country.as_str()) {
                parts.push(country.as_str());
            }
        }
        
        parts.join(", ")
    }
}

pub struct LocationEngine {
    // Map of name -> Vec of possible locations (sorted by population DESC)
    pub cities: HashMap<String, Vec<GeoName>>,
    pub regions: HashMap<String, String>, // "US.CA" -> "California"
    pub countries: HashMap<String, String>, // "US" -> "United States"
    
    // Optimized lookups for O(1) resolution
    country_lookup: HashMap<String, (String, String)>, // normalised name/code -> (code, name)
    region_lookup: HashMap<String, (String, String)>,  // normalised country_code.name/code -> (id, name)
    admin1_lookup: HashMap<String, String>,            // normalised region code -> country code (e.g., "tx" -> "US")

    // compiled regex for keyword removal
    keyword_regex: Regex,
}

#[derive(Clone, Debug)]
pub struct GeoName {
    pub name: String,
    pub country_code: String,
    pub population: u32,
    pub admin1: String,
}

impl LocationEngine {
    pub fn new() -> Self {
        let pattern = format!(r"\b({}|{})\b", 
            REMOTE_KEYWORDS.join("|"), 
            HYBRID_KEYWORDS.join("|")
        );

        Self {
            cities: HashMap::new(),
            regions: HashMap::new(),
            countries: HashMap::new(),
            country_lookup: HashMap::new(),
            region_lookup: HashMap::new(),
            admin1_lookup: HashMap::new(),
            keyword_regex: Regex::new(&pattern).expect("Invalid regex pattern"),
        }
    }

    pub fn load_geonames(&mut self, cities_path: &str, admin_path: &str, country_path: &str) -> Result<()> {
        info!("Loading location data...");
        
        // Load Country Info
        info!("Loading countries...");
        let file = File::open(country_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') { continue; }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 5 { continue; }
            
            let code = parts[0].to_string();
            let name = parts[4].to_string();
            
            // Build fast lookups
            self.country_lookup.insert(code.to_lowercase(), (code.clone(), name.clone()));
            self.country_lookup.insert(name.to_lowercase(), (code.clone(), name.clone()));
            
            self.countries.insert(code, name);
        }
        
        // Add common aliases
        self.country_lookup.insert("usa".to_string(), ("US".to_string(), "United States".to_string()));
        self.country_lookup.insert("uk".to_string(), ("GB".to_string(), "United Kingdom".to_string()));

        // Load Admin1 Codes
        info!("Loading regions...");
        let file = File::open(admin_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 { continue; }
            
            let id = parts[0].to_string(); // e.g., "US.CA"
            let name = parts[1].to_string();
            
            let id_parts: Vec<&str> = id.split('.').collect();
            if id_parts.len() == 2 {
                let country_code = id_parts[0].to_lowercase();
                let region_code = id_parts[1].to_lowercase();
                
                // Composite keys for unambiguous lookups
                self.region_lookup.insert(format!("{}.{}", country_code, region_code), (id.clone(), name.clone()));
                self.region_lookup.insert(format!("{}.{}", country_code, name.to_lowercase()), (id.clone(), name.clone()));

                // Add to admin1 lookup (heuristic: prioritize US or first seen)
                if country_code == "us" || !self.admin1_lookup.contains_key(&region_code) {
                    self.admin1_lookup.insert(region_code, id_parts[0].to_string());
                    // Also map the full name (e.g., "texas" -> "US")
                    self.admin1_lookup.insert(name.to_lowercase(), id_parts[0].to_string());
                }
            }
            
            self.regions.insert(id, name);
        }

        // Load Cities
        info!("Loading cities (this may take a few seconds)...");
        let file = File::open(cities_path)?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 15 { continue; }

            let original_name = parts[1];
            let name_lower = original_name.to_lowercase();
            let asciiname_lower = parts[2].to_lowercase();
            let country_code = parts[8].to_string();
            let population: u32 = parts[14].parse().unwrap_or(0);
            let admin1 = parts[10].to_string();

            let entry = GeoName {
                name: original_name.to_string(),
                country_code,
                population,
                admin1,
            };

            self.cities.entry(name_lower.clone()).or_default().push(entry.clone());
            if asciiname_lower != name_lower {
                 self.cities.entry(asciiname_lower).or_default().push(entry);
            }
            count += 1;
        }

        // Sort by population
        info!("Finalizing city data index...");
        for entries in self.cities.values_mut() {
            entries.sort_by(|a, b| b.population.cmp(&a.population));
        }

        info!("Location engine ready (loaded {} cities).", count);
        Ok(())
    }

    pub fn resolve(&self, raw: &str) -> LocationInfo {
        let (raw_clean, work_mode) = self.extract_work_mode_and_clean(raw);

        if raw_clean.is_empty() {
             return LocationInfo { city: None, region: None, country: None, country_code: None, work_mode };
        }

        // Split on comma, pipe, or slash
        let parts: Vec<&str> = raw_clean.split(|c| c == ',' || c == '|' || c == '/')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // Strategy: Process from most specific to least specific
        let country_found = self.identify_country(&parts);
        let region_found = self.identify_region(&parts, &country_found);
        
        if let Some(location) = self.identify_city(&parts, &country_found, &region_found, work_mode) {
             return location;
        }

        // Fallback for Region/Country only
        self.create_fallback_location(country_found, region_found, work_mode, &parts)
    }

    fn extract_work_mode_and_clean(&self, raw: &str) -> (String, WorkMode) {
        let mut raw_clean = raw.to_lowercase();
        let mut work_mode = WorkMode::InOffice;

        // Check for keywords and remove them in a single pass to ensure consistency
        let mut detected_remote = false;
        let mut detected_hybrid = false;

        raw_clean = self.keyword_regex.replace_all(&raw_clean, |caps: &regex::Captures| {
            let s = caps.get(0).unwrap().as_str();
            if REMOTE_KEYWORDS.contains(&s) {
                detected_remote = true;
            } else if HYBRID_KEYWORDS.contains(&s) {
                detected_hybrid = true;
            }
            ""
        }).to_string();

        if detected_remote {
            work_mode = WorkMode::Remote;
        } else if detected_hybrid {
            work_mode = WorkMode::Hybrid;
        }

        // Clean leading/trailing separators
        raw_clean = raw_clean.trim_matches(|c: char| (!c.is_alphanumeric() && c != ' ') || c.is_whitespace()).to_string();
        
        if raw_clean.starts_with("or ") { raw_clean = raw_clean[3..].trim().to_string(); }
        else if raw_clean.starts_with("and ") { raw_clean = raw_clean[4..].trim().to_string(); }

        (raw_clean, work_mode)
    }

    fn identify_country(&self, parts: &[&str]) -> Option<(String, String)> {
        if let Some(last_part) = parts.last() {
            if let Some(found) = self.country_lookup.get(*last_part) {
                return Some(found.clone());
            }
        }
        None
    }

    fn identify_region(&self, parts: &[&str], country_found: &Option<(String, String)>) -> Option<(String, String)> {
        // Check country context first; else check last part
        let idx = if country_found.is_some() {
             if parts.len() >= 2 { Some(parts.len() - 2) } else { None }
        } else {
             if parts.len() >= 1 { Some(parts.len() - 1) } else { None }
        }?;

        let part = parts[idx];
        
        if let Some((c_code, _)) = country_found {
             // Explicit country context
            let key = format!("{}.{}", c_code.to_lowercase(), part);
            if let Some(found) = self.region_lookup.get(&key) {
                return Some(found.clone());
            }
        } else {
            // Infer country from region code
            if let Some(inferred_cc) = self.admin1_lookup.get(part) {
                 let key = format!("{}.{}", inferred_cc.to_lowercase(), part);
                 if let Some(found) = self.region_lookup.get(&key) {
                     return Some(found.clone());
                 }
            }
        }
        None
    }

    fn identify_city(&self, parts: &[&str], country_found: &Option<(String, String)>, region_found: &Option<(String, String)>, work_mode: WorkMode) -> Option<LocationInfo> {
        // Determine which part to check for city
        let city_part_idx = if region_found.is_some() && country_found.is_none() {
            // Case: Paris, TX -> matches TX. City is at index 0 (len-2).
            if parts.len() >= 2 { Some(parts.len() - 2) } else { None }
        } else {
             // Standard left-most part
             parts.first().map(|_| 0)
        };

        if let Some(idx) = city_part_idx {
            let city_part = parts[idx];
            if let Some(matches) = self.cities.get(city_part) {
                let best = matches.iter().find(|m| {
                    if let Some((c_code, _)) = country_found {
                        if m.country_code != *c_code { return false; }
                    }
                    if let Some((r_id, _)) = region_found {
                        let region_key = format!("{}.{}", m.country_code, m.admin1);
                        if region_key != *r_id { return false; }
                    }
                    true
                }).unwrap_or(&matches[0]);

                let region_key = format!("{}.{}", best.country_code, best.admin1);
                return Some(LocationInfo {
                    city: Some(best.name.clone()),
                    region: self.regions.get(&region_key).cloned(),
                    country: self.countries.get(&best.country_code).cloned(),
                    country_code: Some(best.country_code.clone()),
                    work_mode,
                });
            }
        }
        None
    }

    fn create_fallback_location(&self, mut country_found: Option<(String, String)>, region_found: Option<(String, String)>, work_mode: WorkMode, parts: &[&str]) -> LocationInfo {
        if region_found.is_some() || country_found.is_some() {
             // If we have a region but no country, try to infer country from region
             if country_found.is_none() {
                if let Some((ref r_id, _)) = region_found {
                    let code = r_id.split('.').next().unwrap_or("").to_string();
                    if let Some(name) = self.countries.get(&code) {
                         country_found = Some((code, name.clone()));
                    }
                }
             }

            let (c_code, c_name) = country_found.unwrap_or((String::new(), String::new()));

            return LocationInfo {
                city: None,
                region: region_found.map(|(_, name)| name),
                country: if c_name.is_empty() { None } else { Some(c_name) },
                country_code: if c_code.is_empty() { None } else { Some(c_code) },
                work_mode,
            };
        }

        // Token-based fallback search (if no structure matched)
        for part in parts {
            for token in part.split_whitespace() {
                if let Some(matches) = self.cities.get(token) {
                     let best = &matches[0];
                     let region_key = format!("{}.{}", best.country_code, best.admin1);
                     return LocationInfo {
                         city: Some(best.name.clone()),
                         region: self.regions.get(&region_key).cloned(),
                         country: self.countries.get(&best.country_code).cloned(),
                         country_code: Some(best.country_code.clone()),
                         work_mode,
                     };
                }
            }
        }

        LocationInfo { city: None, region: None, country: None, country_code: None, work_mode }
    }

    #[cfg(test)]
    pub fn new_mock() -> Self {
        let mut engine = Self::new();
        engine.countries.insert("US".to_string(), "United States".to_string());
        engine.country_lookup.insert("us".to_string(), ("US".to_string(), "United States".to_string()));
        engine.country_lookup.insert("united states".to_string(), ("US".to_string(), "United States".to_string()));
        engine.country_lookup.insert("usa".to_string(), ("US".to_string(), "United States".to_string()));
        
        engine.regions.insert("US.CA".to_string(), "California".to_string());
        engine.region_lookup.insert("us.ca".to_string(), ("US.CA".to_string(), "California".to_string()));
        engine.region_lookup.insert("us.california".to_string(), ("US.CA".to_string(), "California".to_string()));
        
        engine.cities.insert("san jose".to_string(), vec![GeoName {
            name: "San Jose".to_string(),
            country_code: "US".to_string(),
            population: 1000000,
            admin1: "CA".to_string(),
        }]);
        
        engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mock() {
        let mut engine = LocationEngine::new_mock();
        // Add manual admin1 lookup for mock since we don't load files in mock
        engine.admin1_lookup.insert("ca".to_string(), "US".to_string());
        engine.admin1_lookup.insert("california".to_string(), "US".to_string());
        // For testing inference from full name
        engine.admin1_lookup.insert("texas".to_string(), "US".to_string());
        engine.regions.insert("US.TX".to_string(), "Texas".to_string());
        engine.region_lookup.insert("us.texas".to_string(), ("US.TX".to_string(), "Texas".to_string()));

        let loc = engine.resolve("San Jose, California, US");
        assert_eq!(loc.city.as_deref(), Some("San Jose"));
        assert_eq!(loc.country_code.as_deref(), Some("US"));
        assert_eq!(loc.display_format(), "San Jose, California, United States");

        // Test "Region, Country" inference (Paris, TX style but with mock data)
        // Mock has San Jose, CA. Let's try "San Jose, CA" without US.
        let loc = engine.resolve("San Jose, CA");
        assert_eq!(loc.city.as_deref(), Some("San Jose"));
        assert_eq!(loc.country_code.as_deref(), Some("US"));
        assert_eq!(loc.region.as_deref(), Some("California"));

        // Test with different delimiter
        let loc = engine.resolve("San Jose / CA / US");
        assert_eq!(loc.city.as_deref(), Some("San Jose"));
        assert_eq!(loc.country_code.as_deref(), Some("US"));


        let loc = engine.resolve("Remote - San Jose");
        assert_eq!(loc.work_mode, WorkMode::Remote);
        assert_eq!(loc.city.as_deref(), Some("San Jose"));

        let loc = engine.resolve("Hybrid");
        assert_eq!(loc.work_mode, WorkMode::Hybrid);
        assert!(loc.city.is_none());

        // Edge case: Ensure partial matches aren't destroyed
        let loc = engine.resolve("Remote, San Jose, CA");  
        assert_eq!(loc.work_mode, WorkMode::Remote);
        assert_eq!(loc.city.as_deref(), Some("San Jose"));
        assert_eq!(loc.region.as_deref(), Some("California"));

        // Test Region Name Inference (Paris, Texas)
        let loc = engine.resolve("Paris, Texas");
        assert_eq!(loc.country_code.as_deref(), Some("US"));
        assert_eq!(loc.region.as_deref(), Some("Texas"));
    }

    #[test]
    fn test_display_format_redundancy() {
        let loc = LocationInfo {
            city: Some("Singapore".to_string()),
            region: Some("Singapore".to_string()),
            country: Some("Singapore".to_string()),
            country_code: Some("SG".to_string()),
            work_mode: WorkMode::InOffice,
        };
        assert_eq!(loc.display_format(), "Singapore");

        let loc = LocationInfo {
            city: Some("New York".to_string()),
            region: Some("New York".to_string()),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            work_mode: WorkMode::InOffice,
        };
        assert_eq!(loc.display_format(), "New York, United States");
    }
}
