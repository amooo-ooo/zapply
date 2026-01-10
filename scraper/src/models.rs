use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum WorkMode {
    Remote,
    Hybrid,
    InOffice,
}

impl Default for WorkMode {
    fn default() -> Self {
        Self::InOffice
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AtsType {
    #[serde(alias = "Greenhouse")]
    Greenhouse,
    #[serde(alias = "Lever")]
    Lever,
    #[serde(alias = "SmartRecruiters")]
    SmartRecruiters,
    #[serde(alias = "Ashby")]
    Ashby,
    #[serde(alias = "Workable")]
    Workable,
    #[serde(alias = "Recruitee")]
    Recruitee,
    #[serde(alias = "Breezy")]
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
    pub domain: Option<String>,
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
    pub company_url: Option<String>,
    pub location: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
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
    pub location: Option<Value>, // Changed from Option<GreenhouseLocation>
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
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersResponse {
    pub content: Vec<SmartRecruitersJob>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersJob {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub released_date: Option<String>,
    pub location: SmartRecruitersLocation,
    pub department: Option<SmartRecruitersLabel>,
    pub company: Option<SmartRecruitersCompany>,
    pub industry: Option<SmartRecruitersIdLabel>,
    pub function: Option<SmartRecruitersIdLabel>,
    pub type_of_employment: Option<SmartRecruitersIdLabel>,
    pub experience_level: Option<SmartRecruitersIdLabel>,
    pub custom_field: Option<Vec<SmartRecruitersCustomField>>,
    pub posting_url: Option<String>,
    pub apply_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersLocation {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub remote: Option<bool>,
    pub hybrid: Option<bool>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub full_location: Option<String>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersLabel {
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersCompany {
    pub name: String,
    pub identifier: String,
}

#[derive(Deserialize)]
pub struct SmartRecruitersIdLabel {
    pub id: Option<String>,
    pub label: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersCustomField {
    pub field_id: String,
    pub field_label: String,
    pub value_id: Option<String>,
    pub value_label: Option<String>,
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
    pub location: Option<Value>, // Changed from Option<String>
    pub published_at: Option<String>,
    pub department: Option<String>,
    pub description_html: Option<AtsDescription>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersDetail {
    pub job_ad: SmartRecruitersJobAd,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersJobAd {
    pub sections: SmartRecruitersSections,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartRecruitersSections {
    pub company_description: Option<SmartRecruitersSection>,
    pub job_description: Option<SmartRecruitersSection>,
    pub qualifications: Option<SmartRecruitersSection>,
    pub additional_information: Option<SmartRecruitersSection>,
}

#[derive(Deserialize)]
pub struct SmartRecruitersSection {
    pub title: Option<String>,
    pub text: Option<String>,
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
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub benefits: Option<String>,
}

#[derive(Deserialize)]
pub struct WorkableDetail {
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub benefits: Option<String>,
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
pub struct RecruiteeDetailResponse {
    pub offer: RecruiteeOfferDetail,
}

#[derive(Deserialize)]
pub struct RecruiteeOfferDetail {
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub benefits: Option<String>,
}

#[derive(Deserialize)]
pub struct BreezyJob {
    pub id: String,
    pub friendly_id: Option<String>,
    pub name: String,
    pub url: Option<String>,
    pub published_date: Option<String>,
    #[serde(rename = "type")]
    pub employment_type: Option<BreezyType>,
    pub location: Option<BreezyLocation>,
    pub department: Option<String>,
    pub salary: Option<String>,
    pub company: Option<BreezyCompany>,
}

#[derive(Deserialize)]
pub struct BreezyType {
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct BreezyCompany {
    pub name: String,
}

#[derive(Deserialize)]
pub struct BreezyLocation {
    pub name: Option<String>,
    pub country: Option<BreezyLabel>,
    pub is_remote: Option<bool>,
    pub remote_details: Option<BreezyLabel>,
}

#[derive(Deserialize)]
pub struct BreezyLabel {
    pub name: Option<String>,
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct BreezyLdJson {
    pub description: Option<String>,
}


