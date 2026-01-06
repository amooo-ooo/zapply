use regex::RegexSet;


pub struct TagEngine {
    regex_set: RegexSet,
    rules: Vec<TagRule>,
}

struct TagRule {
    regex: regex::Regex,
    tag: &'static str,
    /// Optional context requirement (e.g. "Go" needs "language").
    context: Option<regex::Regex>,
    max_word_distance: Option<usize>,
    /// Optional forbidden context (e.g. "Java" but not "Script").
    forbidden_context: Option<regex::Regex>,
    forbidden_max_distance: Option<usize>,
}

impl TagEngine {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        let mut rules = Vec::new();

        macro_rules! add_rule {
            ($pattern:expr, $tag:expr, $ctx:expr, $dist:expr, $forbid:expr, $fdist:expr) => {
                let pat_str = $pattern;
                patterns.push(pat_str.to_string());
                
                let re = regex::RegexBuilder::new(pat_str)
                    .case_insensitive(true)
                    .build()
                    .expect("Invalid keyword regex");

                rules.push(TagRule {
                    regex: re,
                    tag: $tag,
                    context: $ctx,
                    max_word_distance: $dist,
                    forbidden_context: $forbid,
                    forbidden_max_distance: $fdist,
                });
            };
        }


        macro_rules! simple { 
            ($p:expr, $t:expr) => { add_rule!($p, $t, None, None, None, None) } 
        }
        
        macro_rules! strict_dist {
            ($p:expr, $t:expr, $ctx:expr, $d:expr) => {
                let ctx_re = regex::RegexBuilder::new($ctx).case_insensitive(true).build().expect("Invalid context regex");
                add_rule!($p, $t, Some(ctx_re), Some($d), None, None)
            }
        }

        // === Software Engineering ===
        simple!(r"(?i)\brust\b", "Rust");
        simple!(r"(?i)\bpython\b", "Python");
        simple!(r"(?i)\bjavascript\b|(^|[^.])\bjs\b", "JavaScript");
        simple!(r"(?i)\btypescript\b|(^|[^.])\bts\b", "TypeScript");
        simple!(r"(?i)\bgolang\b", "Go");
        strict_dist!(r"(?i)\bgo\b", "Go", r"(?i)\blanguage\b", 5);
        
        simple!(r"(?i)\bjava\b", "Java");
        simple!(r"(?i)\bc\+\+\b", "C++");
        simple!(r"(?i)\bc#\b", "C#");
        simple!(r"(?i)\bruby\b", "Ruby");
        simple!(r"(?i)\bphp\b", "PHP");
        simple!(r"(?i)\bswift\b", "Swift");
        simple!(r"(?i)\bkotlin\b", "Kotlin");
        simple!(r"(?i)\bscala\b", "Scala");
        simple!(r"(?i)\belixir\b", "Elixir");
        
        // Frameworks & Libraries
        simple!(r"(?i)\breact\b", "React");
        simple!(r"(?i)\bvue\b", "Vue");
        simple!(r"(?i)\bangular\b", "Angular");
        simple!(r"(?i)\bsvelte\b", "Svelte");
        simple!(r"(?i)\bnext\.?js\b", "Next.js");
        simple!(r"(?i)\bnuxt\b", "Nuxt");
        simple!(r"(?i)\bnode\.?js\b", "Node.js");
        simple!(r"(?i)\bdjango\b", "Django");
        simple!(r"(?i)\bflask\b", "Flask");
        simple!(r"(?i)\bfastapi\b", "FastAPI");
        simple!(r"(?i)\bspring\b", "Spring");
        simple!(r"(?i)\.net\b", ".NET");
        simple!(r"(?i)\brails\b", "Ruby on Rails");
        simple!(r"(?i)\blaravel\b", "Laravel");
        simple!(r"(?i)\btailwind\b", "Tailwind");
        simple!(r"(?i)\btensorflow\b", "TensorFlow");
        simple!(r"(?i)\bpytorch\b", "PyTorch");

        // Infrastructure & Tools
        simple!(r"(?i)\bdocker\b", "Docker");
        simple!(r"(?i)\bkubernetes\b|k8s\b", "Kubernetes");
        simple!(r"(?i)\baws\b", "AWS");
        simple!(r"(?i)\bazure\b", "Azure");
        simple!(r"(?i)\bgcp\b|google cloud\b", "GCP");
        simple!(r"(?i)\bterraform\b", "Terraform");
        simple!(r"(?i)\blinux\b", "Linux");
        simple!(r"(?i)\bgit\b", "Git");
        simple!(r"(?i)\bsql\b", "SQL");
        simple!(r"(?i)\bnosql\b", "NoSQL");
        simple!(r"(?i)\bredis\b", "Redis");
        simple!(r"(?i)\bkafka\b", "Kafka");
        simple!(r"(?i)\bgraphql\b", "GraphQL");
        simple!(r"(?i)\brest\b", "REST");

        // === Data & Analytics ===
        simple!(r"(?i)\bdata scien(ce|tist)\b", "Data Science");
        simple!(r"(?i)\bmachine learning\b|\bml\b", "Machine Learning");
        simple!(r"(?i)\bartificial intelligence\b|\bai\b", "AI");
        simple!(r"(?i)\bnlp\b", "NLP");
        simple!(r"(?i)\bstatistics\b", "Statistics");
        simple!(r"(?i)\bpandas\b", "Pandas");
        simple!(r"(?i)\bnumpy\b", "NumPy");
        simple!(r"(?i)\btableau\b", "Tableau");
        simple!(r"(?i)\bpower bi\b", "Power BI");
        simple!(r"(?i)\bsql server\b", "SQL Server");
        simple!(r"(?i)\bpostgresql\b|\bpostgres\b", "PostgreSQL");

        // === Product & Design ===
        simple!(r"(?i)\bproduct manage(r|ment)\b|\bpm\b", "Product Management");
        simple!(r"(?i)\bproduct owner\b", "Product Owner");
        simple!(r"(?i)\bui\b|\buser interface\b", "UI");
        simple!(r"(?i)\bux\b|\buser experience\b", "UX");
        simple!(r"(?i)\bfigma\b", "Figma");
        simple!(r"(?i)\bsketch\b", "Sketch");
        simple!(r"(?i)\bgraphic design\b", "Graphic Design");

        // === Marketing & Sales (Strict) ===
        // Must be associated with role-specific keywords, not just company description
        strict_dist!(r"(?i)\bseo\b", "SEO", r"(?i)\b(specialist|optimization|ranking|keyword|content|audit|technical)\b", 15);
        strict_dist!(r"(?i)\bsem\b", "SEM", r"(?i)\b(paid|search|marketing|campaign|ppc|ad)\b", 15);
        simple!(r"(?i)\bcontent marketing\b", "Content Marketing");
        simple!(r"(?i)\bcopywriting\b", "Copywriting");
        simple!(r"(?i)\bsocial media\b", "Social Media");
        simple!(r"(?i)\bbusiness development\b|\bbdr\b|\bsdr\b", "Business Development");
        simple!(r"(?i)\baccount manage(r|ment)\b", "Account Management");
        simple!(r"(?i)\bcrm\b", "CRM");
        simple!(r"(?i)\bsalesforce\b", "Salesforce");
        strict_dist!(r"(?i)\bugc\b|user generated content\b", "UGC", r"(?i)\b(marketing|content|campaign|social|creator)\b", 15);
        strict_dist!(r"(?i)\bcro\b|conversion rate optimization\b", "CRO", r"(?i)\b(optimization|experiment|testing|growth|marketing)\b", 15);
        strict_dist!(r"(?i)\bppc\b|pay[-\s]per[-\s]click\b", "PPC", r"(?i)\b(campaign|ad|paid|marketing|search)\b", 15);
        strict_dist!(r"(?i)\bgtm\b|go[-\s]to[-\s]market\b", "Go-to-Market", r"(?i)\b(launch|product|market|sales)\b", 15);
        
        strict_dist!(r"(?i)\bb2b\b", "B2B", r"(?i)\b(sales|marketing|saas|client|account|business)\b", 15);
        strict_dist!(r"(?i)\bb2c\b", "B2C", r"(?i)\b(consumer|marketing|sales|brand|customer|retail)\b", 15);
        
        simple!(r"(?i)\binfluencer\b", "Influencer Marketing");
        strict_dist!(r"(?i)\baffiliate\b", "Affiliate Marketing", r"(?i)\b(program|marketing|network|partner)\b", 15);

        // === Finance & Accounting (Strict) ===
        strict_dist!(r"(?i)\baccounting\b", "Accounting", r"(?i)\b(staff|clerk|financial|ledger|payable|receivable|reconciliation|cpa|intern)\b", 15);
        simple!(r"(?i)\bcpa\b", "CPA");
        strict_dist!(r"(?i)\baudit\b", "Audit", r"(?i)\b(internal|external|financial|risk|compliance|it|process|assurance)\b", 15);
        strict_dist!(r"(?i)\btax\b", "Tax", r"(?i)\b(compliance|return|filing|income|corporate|sales|provision|indirect|salt)\b", 15);
        simple!(r"(?i)\binvestment banking\b", "Investment Banking");
        simple!(r"(?i)\btrading\b", "Trading");
        simple!(r"(?i)\bfp&a\b", "FP&A");
        simple!(r"(?i)\btreasury\b", "Treasury");
        simple!(r"(?i)\bventure capital\b|\bvc\b", "Venture Capital");
        simple!(r"(?i)\bprivate equity\b|\bpe\b", "Private Equity");

        // === Operations & HR ===
        simple!(r"(?i)\bsupply chain\b", "Supply Chain");
        simple!(r"(?i)\blogistics\b", "Logistics");
        simple!(r"(?i)\bproject manage(r|ment)\b", "Project Management");
        simple!(r"(?i)\bprogram manage(r|ment)\b", "Program Management");
        simple!(r"(?i)\bhuman resources\b|\bhr\b", "Human Resources");
        simple!(r"(?i)\brecruiting\b|\brecruiter\b", "Recruiting");
        simple!(r"(?i)\btalent acquisition\b", "Talent Acquisition");
        simple!(r"(?i)\bpeople ops\b", "People Ops");

        // === Legal ===
        strict_dist!(r"(?i)\bcompliance\b", "Compliance", r"(?i)\b(regulatory|legal|risk|policy|standard|gdpr|hipaa|soc2|analyst)\b", 15);
        simple!(r"(?i)\blitigation\b", "Litigation");
        simple!(r"(?i)\bcontract law\b", "Contract Law");
        simple!(r"(?i)\bintellectual property\b|\bip\b", "Intellectual Property");
        simple!(r"(?i)\bparalegal\b", "Paralegal");
        simple!(r"(?i)\battorney\b", "Attorney");

        // === Hardware & Science ===
        simple!(r"(?i)\belectrical engineering\b", "Electrical Engineering");
        simple!(r"(?i)\bmechanical engineering\b", "Mechanical Engineering");
        simple!(r"(?i)\bcivil engineering\b", "Civil Engineering");
        simple!(r"(?i)\bchemical engineering\b", "Chemical Engineering");
        simple!(r"(?i)\bbiomedical\b", "Biomedical");

        // === General & Benefits ===
        simple!(r"(?i)\blgbtq(\+|\b)", "LGBTQ+ Friendly");
        simple!(r"(?i)\bpaid (internship|role|position)\b", "Paid");
        simple!(r"(?i)\bvisa sponsorship\b", "Visa Sponsorship");
        simple!(r"(?i)\bremote\b", "Remote");
        simple!(r"(?i)\bhybrid\b", "Hybrid");

        let regex_set = RegexSet::new(patterns).expect("Failed to create RegexSet");

        Self { regex_set, rules }
    }

    pub fn detect_tags(&self, text: &str) -> Vec<&'static str> {

        let matches = self.regex_set.matches(text);
        
        matches.into_iter()
            .filter_map(|index| {
                let rule = &self.rules[index];
                
                if let Some(context_re) = &rule.context {
                    if !context_re.is_match(text) {
                        return None;
                    }
                    
                    if let Some(max_dist) = rule.max_word_distance {
                        if !self.check_distance(text, &rule.regex, context_re, max_dist, true) {
                            return None;
                        }
                    }
                }
                
                if let Some(forbidden_re) = &rule.forbidden_context {
                    if forbidden_re.is_match(text) {
                        if let Some(forbidden_dist) = rule.forbidden_max_distance {
                             if self.check_distance(text, &rule.regex, forbidden_re, forbidden_dist, true) {
                                 return None;
                             }
                        } else {
                            return None;
                        }
                    }
                }
                
                Some(rule.tag)
            })
            .collect()
    }
    
    fn check_distance(&self, text: &str, keyword_re: &regex::Regex, context_re: &regex::Regex, max_dist: usize, _match_must_exist: bool) -> bool {
        let keyword_indices: Vec<usize> = keyword_re.find_iter(text).map(|m| m.start()).collect();
        let context_indices: Vec<usize> = context_re.find_iter(text).map(|m| m.start()).collect();
        
        for &k_idx in &keyword_indices {
            for &c_idx in &context_indices {
                let (start, end) = if k_idx < c_idx { (k_idx, c_idx) } else { (c_idx, k_idx) };
                let slice = &text[start..end];

                if count_words(slice) <= max_dist {
                    return true;
                }
            }
        }
        false
    }
}

fn count_words(s: &str) -> usize {
    let mut count = 0;
    let mut in_word = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if in_word {
                count += 1;
                in_word = false;
            }
        } else {
            in_word = true;
        }
    }
    count
}

// === Education Detection ===

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EducationInfo {
    pub degree_levels: Vec<String>,
    pub subject_areas: Vec<String>,
}

pub struct EducationDetector {
    regex_set: regex::RegexSet,
    rules: Vec<EducationRule>,
    context_regex: regex::Regex,
}

struct EducationRule {
    tag: &'static str,
    kind: EducationKind,
}

enum EducationKind {
    Degree,
    Subject,
}

impl EducationDetector {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        let mut rules = Vec::new();

        macro_rules! add_edu {
            ($p:expr, $t:expr, $k:expr) => {
                patterns.push($p.to_string());
                rules.push(EducationRule {
                    tag: $t,
                    kind: $k,
                });
            };
        }

        macro_rules! degree {
            ($p:expr, $t:expr) => { add_edu!($p, $t, EducationKind::Degree) }
        }

        macro_rules! subject {
            ($p:expr, $t:expr) => { add_edu!($p, $t, EducationKind::Subject) }
        }

        // Degree levels
        degree!(r"\b(bachelor'?s?|b\.?s\.?|b\.?a\.?|bsc|ba)\b", "Bachelor's");
        degree!(r"\b(master'?s?|m\.?s\.?|m\.?a\.?|msc|ma|mba)\b", "Master's");
        degree!(r"\b(ph\.?d\.?|doctorate|doctoral)\b", "PhD");
        degree!(r"\b(associate'?s?|a\.?s\.?|a\.?a\.?)\b", "Associate's");

        // Subject areas
        subject!(r"\b(computer science|cs)\b", "Computer Science");
        subject!(r"\b(software engineering)\b", "Software Engineering");
        subject!(r"\b(business informatics|wirtschaftsinformatik)\b", "Business Informatics");
        subject!(r"\binformatics\b", "Informatics");
        subject!(r"\b(information systems|information technology|it)\b", "Information Systems");
        subject!(r"\b(data science)\b", "Data Science");
        subject!(r"\b(artificial intelligence|ai|machine learning)\b", "AI/ML");
        subject!(r"\b(mathematics|math|maths)\b", "Mathematics");
        subject!(r"\b(statistics)\b", "Statistics");
        
        // Business & Economics
        subject!(r"\b(economics)\b", "Economics");
        subject!(r"\b(business administration|bba|business studies)\b", "Business Administration");
        subject!(r"\b(finance)\b", "Finance");
        subject!(r"\b(accounting)\b", "Accounting");
        subject!(r"\b(marketing)\b", "Marketing");
        
        // Engineering
        subject!(r"\b(electrical engineering|ee)\b", "Electrical Engineering");
        subject!(r"\b(mechanical engineering)\b", "Mechanical Engineering");
        subject!(r"\b(engineering)\b", "Engineering");

        let regex_set = regex::RegexSetBuilder::new(patterns)
            .case_insensitive(true)
            .build()
            .expect("Invalid education regex set");

        let context_regex = regex::RegexBuilder::new(
            r"(?i)\b(studying|enrolled|pursuing|degree|student|graduate|graduating|completed|completing|working towards?|currently in|candidate|major|studies)\b"
        )
        .case_insensitive(true)
        .build()
        .expect("Invalid context regex");

        Self {
            regex_set,
            rules,
            context_regex,
        }
    }

    pub fn detect(&self, text: &str) -> EducationInfo {
        if !self.context_regex.is_match(text) {
            return EducationInfo::default();
        }

        let mut info = EducationInfo::default();
        let matches = self.regex_set.matches(text);

        for index in matches {
            let rule = &self.rules[index];
            match rule.kind {
                EducationKind::Degree => {
                    if !info.degree_levels.contains(&rule.tag.to_string()) {
                        info.degree_levels.push(rule.tag.to_string());
                    }
                }
                EducationKind::Subject => {
                    if !info.subject_areas.contains(&rule.tag.to_string()) {
                        info.subject_areas.push(rule.tag.to_string());
                    }
                }
            }
        }

        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_detect_tags() {
        let engine = TagEngine::new();
        let text = "We are looking for a Rust developer who knows Python and Docker. Experience with Next.js is a plus.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();
        
        assert!(tags_set.contains("Rust"));
        assert!(tags_set.contains("Python"));
        assert!(tags_set.contains("Docker"));
        assert!(tags_set.contains("Next.js"));
        assert_eq!(tags.len(), 4);
    }
    
    #[test]
    fn test_case_insensitive() {
        let engine = TagEngine::new();
        let tags = engine.detect_tags("react node.js Golang");
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("React"));
        assert!(tags_set.contains("Node.js"));
        assert!(tags_set.contains("Go"));
    }

    #[test]
    fn test_word_boundaries() {
        let engine = TagEngine::new();
        let tags = engine.detect_tags("I like running fast. reaction.");
        assert!(!tags.contains(&"React"));
    }

    #[test]
    fn test_multidisciplinary_tags() {
        let engine = TagEngine::new();
        let text = "We need a Product Manager who knows SQL and has experience with Accounting reconciliation and FP&A models.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("Product Management"));
        assert!(tags_set.contains("SQL"));
        assert!(tags_set.contains("Accounting"));
        assert!(tags_set.contains("FP&A"));
    }

    #[test]
    fn test_general_tags() {
        let engine = TagEngine::new();
        let text = "Paid internship. LGBTQ+ friendly. Visa sponsorship. Remote work.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("Paid"));
        assert!(tags_set.contains("LGBTQ+ Friendly"));
        assert!(tags_set.contains("Visa Sponsorship"));
        assert!(tags_set.contains("Remote"));
    }

    #[test]
    fn test_marketing_jargon() {
        let engine = TagEngine::new();
        let text = "B2B Marketing Specialist with PPC, SEO optimization, and Go-to-Market launch strategies.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("B2B"));
        assert!(tags_set.contains("PPC"));
        assert!(tags_set.contains("SEO"));
        assert!(tags_set.contains("Go-to-Market"));
    }

    #[test]
    fn test_strict_go_rule() {
        let engine = TagEngine::new();
        assert!(engine.detect_tags("Looking for a Golang developer").contains(&"Go"));
        assert!(engine.detect_tags("Must know the Go programming language").contains(&"Go"));
        
        let far_text = "我们 Go to the store to buy some milk and bread and then verify the programming language syntax.";
        assert!(!engine.detect_tags(far_text).contains(&"Go"));
        
        let tags = engine.detect_tags("We go fast here");
        assert!(!tags.contains(&"Go"));
    }

    #[test]
    fn test_strict_generic_tags() {
        let engine = TagEngine::new();
        
        // --- B2B ---
        // False positive scenario: Company description
        let b2b_desc = "We are a B2B company focused on excellence.";
        assert!(!engine.detect_tags(b2b_desc).contains(&"B2B"));
        
        // True positive scenario: Job requirement
        let b2b_job = "Looking for a B2B Sales Associate to drive growth.";
        assert!(engine.detect_tags(b2b_job).contains(&"B2B"));

        // --- SEO ---
        // False: Company description
        let seo_company = "Our company specializes in SEO services.";
        assert!(!engine.detect_tags(seo_company).contains(&"SEO")); 
        
        // True: Job title/role
        let seo_job = "Hiring an SEO Specialist to improve our rankings.";
        assert!(engine.detect_tags(seo_job).contains(&"SEO"));

       // --- Accounting ---
       let acc_desc = "We are a leading Accounting firm.";
       assert!(!engine.detect_tags(acc_desc).contains(&"Accounting"));
       
       // "Senior Accounting Manager" would fail now, so we test "Staff Accountant" or "Intern"
       let acc_job = "We need a Staff Accounting Clerk for our finance team.";
       assert!(engine.detect_tags(acc_job).contains(&"Accounting"));
    }

    #[test]
    fn test_manual_negative_context() {
        // Manually test the logic that would be used for negative context
        let mut patterns = Vec::new();
        let mut rules = Vec::new();
        
        let pat_str = r"(?i)\bjava\b";
        patterns.push(pat_str.to_string());
        
        let context_re: Option<regex::Regex> = None;

        
        rules.push(TagRule {
            regex: regex::RegexBuilder::new(pat_str).case_insensitive(true).build().unwrap(),
            tag: "Java",
            context: context_re,
            max_word_distance: None,
            forbidden_context: Some(regex::RegexBuilder::new(r"(?i)\bscript\b").case_insensitive(true).build().unwrap()),
            forbidden_max_distance: Some(1),
        });
        
        let engine = TagEngine {
            regex_set: RegexSet::new(patterns).unwrap(),
            rules,
        };
        
        assert!(engine.detect_tags("I know Java well.").contains(&"Java"));
        // "Java Script"
        assert!(!engine.detect_tags("I know Java Script.").contains(&"Java"));
    }

    // === Education Detection Tests ===

    #[test]
    fn test_education_degree_level() {
        let detector = EducationDetector::new();
        
        // Bachelor's with context
        let info = detector.detect("Currently enrolled in Bachelor's degree program");
        assert!(info.degree_levels.contains(&"Bachelor's".to_string()));
        
        // Master's with context
        let info = detector.detect("Pursuing a Master's in Computer Science");
        assert!(info.degree_levels.contains(&"Master's".to_string()));
        
        // PhD
        let info = detector.detect("Ph.D. candidate in Data Science");
        assert!(info.degree_levels.contains(&"PhD".to_string()));
    }

    #[test]
    fn test_education_subject_area() {
        let detector = EducationDetector::new();
        
        // Computer Science
        let info = detector.detect("Student studying Computer Science");
        assert!(info.subject_areas.contains(&"Computer Science".to_string()));
        
        // Business Informatics
        let info = detector.detect("Enrolled in Business Informatics degree");
        assert!(info.subject_areas.contains(&"Business Informatics".to_string()));
        
        // Informatics
        let info = detector.detect("Pursuing studies in Informatics");
        assert!(info.subject_areas.contains(&"Informatics".to_string()));
    }

    #[test]
    fn test_education_combined() {
        let detector = EducationDetector::new();
        
        // Both degree and subject
        let info = detector.detect("Currently pursuing a Master's degree in Computer Science");
        assert!(info.degree_levels.contains(&"Master's".to_string()));
        assert!(info.subject_areas.contains(&"Computer Science".to_string()));
    }

    #[test]
    fn test_education_multiple() {
        let detector = EducationDetector::new();
        
        // Multiple subjects
        let info = detector.detect("Studying a degree in Computer Science and Mathematics");
        assert!(info.subject_areas.contains(&"Computer Science".to_string()));
        assert!(info.subject_areas.contains(&"Mathematics".to_string()));

        // Multiple degrees
        let info = detector.detect("Candidate for Bachelor's or Master's in Computer Science");
        assert!(info.degree_levels.contains(&"Bachelor's".to_string()));
        assert!(info.degree_levels.contains(&"Master's".to_string()));
    }

    #[test]
    fn test_education_requires_context() {
        let detector = EducationDetector::new();
        
        // No context = no detection
        let info = detector.detect("We use Computer Science principles here");
        assert!(info.degree_levels.is_empty());
        assert!(info.subject_areas.is_empty());
        
        // With context = detection works
        let info = detector.detect("We require a student studying Computer Science");
        assert!(info.subject_areas.contains(&"Computer Science".to_string()));
    }

    #[test]
    fn test_education_no_false_positives() {
        let detector = EducationDetector::new();
        
        // Random text without education context
        let info = detector.detect("We are a technology company building great products");
        assert_eq!(info, EducationInfo::default());
    }
}
