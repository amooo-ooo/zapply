use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AtsType {
    Greenhouse,
    Lever,
    SmartRecruiters,
    Ashby,
    Workable,
    Recruitee,
    Breezy,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum FlexibleId {
    Number(i64),
    String(String),
}

impl std::fmt::Display for FlexibleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(untagged)]
pub enum AtsDescription {
    String(String),
    Object { value: String },
}

impl AtsDescription {
    pub fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s,
            Self::Object { value } => value,
        }
    }
}


#[derive(Debug, Deserialize, Clone)]
pub struct CompanyEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub ats_type: AtsType,
    pub slug: String,
    pub api_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub title: String,
    pub description: String,
    pub company: String,
    pub slug: String,
    pub ats: AtsType,
    pub url: String,
    pub location: String,
    pub posted: String,
    pub departments: Vec<String>,
    pub offices: Vec<String>,
    pub tags: Vec<String>,
    pub degree_levels: Vec<String>,
    pub subject_areas: Vec<String>,
}

// --- Specialized Response Structs ---

#[derive(Deserialize, Clone)]
pub struct RawGreenhouseJob {
    pub id: FlexibleId,
    pub title: String,
    #[serde(alias = "absolute_url")]
    pub url: String,
    #[serde(alias = "content", alias = "description")]
    pub description: Option<AtsDescription>,
    pub location: Option<GreenhouseLocation>,
    #[serde(alias = "updated_at")]
    pub posted: Option<String>,
    pub education: Option<GreenhouseEducation>,
    pub metadata: Option<Vec<GreenhouseMetadataItem>>,
    #[serde(default)]
    pub departments: Vec<RawGreenhouseNameItem>,
    #[serde(default)]
    pub offices: Vec<RawGreenhouseNameItem>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum GreenhouseLocation {
    Object { name: String },
    String(String),
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum GreenhouseEducation {
    Object { value: String },
    String(String),
}

#[derive(Deserialize, Clone)]
pub struct GreenhouseMetadataItem {
    pub name: Option<String>,
    pub label: Option<String>,
    pub value: Value,
}

#[derive(Deserialize, Clone)]
pub struct RawGreenhouseNameItem {
    pub name: Option<String>,
}



#[derive(Deserialize)]
pub struct LeverJob {
    pub id: String,
    pub text: String,
    pub hosted_url: String,
    #[serde(rename = "descriptionPlain")]
    pub description_plain: Option<String>,
    pub description: Option<String>,
    pub categories: LeverCategories,
    #[serde(rename = "createdAt")]
    pub created_at: Option<u64>,
}

#[derive(Deserialize)]
pub struct LeverCategories {
    pub location: Option<String>,
    pub team: Option<String>,
    pub department: Option<String>,
    pub commitment: Option<String>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersResponse {
    pub content: Vec<SmartRecruitersJob>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersJob {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub released_date: Option<String>,
    pub location: SmartRecruitersLocation,
    pub department: Option<SmartRecruitersLabel>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersLocation {
    pub city: Option<String>,
    pub country: Option<String>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersLabel {
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct AshbyResponse {
    pub jobs: Vec<AshbyJob>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AshbyJob {
    pub id: String,
    pub title: String,
    pub job_url: String,
    pub location: Option<String>,
    pub published_at: Option<String>,
    pub department: Option<String>,
    pub description_plain: Option<AtsDescription>,
    pub description_html: Option<AtsDescription>,
}

#[derive(Deserialize)]
pub struct WorkableResponse {
    pub jobs: Vec<WorkableJob>,
}

#[derive(Deserialize)]
pub struct WorkableJob {
    pub shortcode: String,
    pub title: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Deserialize)]
pub struct RecruiteeResponse {
    pub offers: Vec<RecruiteeJob>,
}

#[derive(Deserialize)]
pub struct RecruiteeJob {
    pub id: u64,
    pub title: String,
    pub careers_url: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub created_at: Option<String>,
    pub department: Option<String>,
}

#[derive(Deserialize)]
pub struct BreezyJob {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<BreezyLocation>,
    pub updated_at: Option<String>,
    pub department: Option<String>,
}

#[derive(Deserialize)]
pub struct BreezyLocation {
    pub name: Option<String>,
}
