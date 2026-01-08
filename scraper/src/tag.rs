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
        simple!(r"(?i)\bhaskell\b", "Haskell");
        simple!(r"(?i)\berlang\b", "Erlang");
        simple!(r"(?i)\bclojure\b", "Clojure");
        
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
        
        // Software Engineering & DevOps
        simple!(r"(?i)\bjenkins\b", "Jenkins");
        simple!(r"(?i)\bgitlab\b", "GitLab");
        simple!(r"(?i)\bgithub actions\b", "GitHub Actions");
        simple!(r"(?i)\bcircleci\b", "CircleCI");
        simple!(r"(?i)\bansible\b", "Ansible");
        simple!(r"(?i)\bpulumi\b", "Pulumi");
        simple!(r"(?i)\bprometheus\b", "Prometheus");
        simple!(r"(?i)\bgrafana\b", "Grafana");
        simple!(r"(?i)\belk stack\b|\belasticsearch\b", "Elasticsearch");
        simple!(r"(?i)\bsplunk\b", "Splunk");
        simple!(r"(?i)\bnginx\b", "NGINX");
        simple!(r"(?i)\bapache\b", "Apache");
        simple!(r"(?i)\bserverless\b", "Serverless");
        simple!(r"(?i)\bcassandra\b", "Cassandra");
        simple!(r"(?i)\bmongodb\b", "MongoDB");
        simple!(r"(?i)\bmariadb\b", "MariaDB");
        strict_dist!(r"(?i)\bsnowflake\b", "Snowflake", r"(?i)\b(data|lake|warehouse|cloud|analytics|sql|computing)\b", 15);
        simple!(r"(?i)\bdatabricks\b", "Databricks");
        simple!(r"(?i)\bbigquery\b", "BigQuery");
        simple!(r"(?i)\bairflow\b", "Airflow");
        simple!(r"(?i)\bdbt\b", "dbt");

        // Telehealth & Health IT
        simple!(r"(?i)\btelehealth\b|\btelemedicine\b", "Telehealth");
        strict_dist!(r"(?i)\bepic\b", "Epic Systems", r"(?i)\b(systems|electronic|health|record|software|ehr|emr|certified|analyst|telehealth|platform)\b", 15);
        simple!(r"(?i)\bcerner\b", "Cerner");
        simple!(r"(?i)\behr\b|\bemr\b", "EHR/EMR");
        simple!(r"(?i)\bhl7\b", "HL7");
        simple!(r"(?i)\bfhir\b", "FHIR");
        simple!(r"(?i)\bdicom\b", "DICOM");
        simple!(r"(?i)\bpacs\b", "PACS");
        simple!(r"(?i)\bpointclickcare\b", "PointClickCare");
        simple!(r"(?i)\bpractice fusion\b", "Practice Fusion");
        strict_dist!(r"(?i)\bhipaa\b", "HIPAA Compliance", r"(?i)\b(compliance|security|privacy|regulation|standards|training)\b", 15);
        simple!(r"(?i)\bmedtech\b", "MedTech");
        simple!(r"(?i)\bbiotech\b", "Biotech");
        simple!(r"(?i)\bbioinformatics\b", "Bioinformatics");
        simple!(r"(?i)\bclinical trials\b", "Clinical Trials");
        simple!(r"(?i)\bpharmacovigilance\b", "Pharmacovigilance");
        
        // HealthTech specifics
        simple!(r"(?i)\bathenahealth\b", "Athenahealth");
        simple!(r"(?i)\ballscripts\b", "Allscripts");
        simple!(r"(?i)\bmeditech\b", "Meditech");
        simple!(r"(?i)\beclinicalworks\b", "eClinicalWorks");
        simple!(r"(?i)\bcarecloud\b", "CareCloud");
        simple!(r"(?i)\bnextgen\b", "NextGen Health");

        // Business Technologies & SaaS
        simple!(r"(?i)\bsap\b", "SAP");
        simple!(r"(?i)\boracle erp\b", "Oracle ERP");
        simple!(r"(?i)\bnetsuite\b", "NetSuite");
        simple!(r"(?i)\bworkday\b", "Workday");
        simple!(r"(?i)\bservicenow\b", "ServiceNow");
        simple!(r"(?i)\bhubspot\b", "HubSpot");
        simple!(r"(?i)\bmarketo\b", "Marketo");
        simple!(r"(?i)\bpardot\b", "Pardot");
        simple!(r"(?i)\bzendesk\b", "Zendesk");
        simple!(r"(?i)\bintercom\b", "Intercom");
        simple!(r"(?i)\bshopify\b", "Shopify");
        simple!(r"(?i)\bmagento\b", "Magento");
        simple!(r"(?i)\bwoo?commerce\b", "WooCommerce");
        simple!(r"(?i)\bslack\b", "Slack");
        simple!(r"(?i)\bmicrosoft teams\b", "MS Teams");
        simple!(r"(?i)\bjira\b", "Jira");
        simple!(r"(?i)\bconfluence\b", "Confluence");
        simple!(r"(?i)\btrello\b", "Trello");
        simple!(r"(?i)\basana\b", "Asana");
        simple!(r"(?i)\bmonday\.com\b", "Monday.com");
        simple!(r"(?i)\bnotion\b", "Notion");
        simple!(r"(?i)\berp\b", "ERP");
        simple!(r"(?i)\bgoogle (suite|workspace|docs|sheets|slides)\b", "Google Workspace");
        simple!(r"(?i)\bmicrosoft (office|excel|word|powerpoint)\b|\bexcel\b|\bpowerpoint\b", "Microsoft Office");

        // Creative & UI/UX specifics
        simple!(r"(?i)\badobe xd\b", "Adobe XD");
        simple!(r"(?i)\bframer\b", "Framer");
        simple!(r"(?i)\bprinciple\b", "Principle");
        simple!(r"(?i)\bzeplin\b", "Zeplin");
        simple!(r"(?i)\binvision\b", "InVision");
        simple!(r"(?i)\bcoreldraw\b", "CorelDraw");

        // Design & Creative
        simple!(r"(?i)\badobe (creative cloud|suite)\b", "Adobe CC");
        simple!(r"(?i)\bphotoshop\b", "Photoshop");
        simple!(r"(?i)\billustrator\b", "Illustrator");
        simple!(r"(?i)\bindesign\b", "InDesign");
        simple!(r"(?i)\bafter effects\b", "After Effects");
        simple!(r"(?i)\bpremiere pro\b", "Premiere Pro");
        simple!(r"(?i)\bcanva\b", "Canva");
        simple!(r"(?i)\bwebflow\b", "Webflow");
        simple!(r"(?i)\bblender\b", "Blender");
        strict_dist!(r"(?i)\bunity(3d)?\b", "Unity", r"(?i)\b(engine|game|developer|developing|design|c#|real[-\s]time|vr|ar)\b", 15);
        simple!(r"(?i)\bunreal engine\b", "Unreal Engine");

        // Engineering & Science
        simple!(r"(?i)\brobotics\b", "Robotics");
        strict_dist!(r"(?i)\bros\b", "ROS", r"(?i)\b(robot|robotics|operating|system|kinematics|navigation|control|developer|simulation)\b", 15);
        strict_dist!(r"(?i)\bcad\b", "CAD", r"(?i)\b(computer|aided|design|software|autocad|solidworks|modelling|drawing|drafting|technical)\b", 15);
        simple!(r"(?i)\bsolidworks\b", "SolidWorks");
        simple!(r"(?i)\bautocad\b", "AutoCAD");
        strict_dist!(r"(?i)\bmatlab\b", "MATLAB", r"(?i)\b(simulation|programming|script|algorithm|signal|processing|mathworks|academic|experience|familiarity)\b", 15);
        simple!(r"(?i)\blabview\b", "LabVIEW");
        strict_dist!(r"(?i)\bfpga\b", "FPGA", r"(?i)\b(design|verilog|vhdl|logic|hardware|circuit|programmable|gate)\b", 15);
        simple!(r"(?i)\bverilog\b", "Verilog");
        simple!(r"(?i)\bvhdl\b", "VHDL");
        strict_dist!(r"(?i)\brtos\b|real[-\s]time operating system\b", "RTOS", r"(?i)\b(embedded|kernel|task|scheduler|interrupt|thread|safety|critical)\b", 15);
        simple!(r"(?i)\bembedded c\b", "Embedded C");
        strict_dist!(r"(?i)\bplc\b|programmable logic controller\b", "PLC", r"(?i)\b(automation|control|industrial|programming|ladder|logic|scada|hmi)\b", 15);
        simple!(r"(?i)\bscada\b", "SCADA");
        simple!(r"(?i)\bansys\b", "ANSYS");

        // Engineering/Industrial specifics
        simple!(r"(?i)\bsolid edge\b", "Solid Edge");
        simple!(r"(?i)\bsiemens nx\b", "Siemens NX");
        simple!(r"(?i)\bcatia\b", "CATIA");
        simple!(r"(?i)\bfusion 360\b", "Fusion 360");
        simple!(r"(?i)\bteamcenter\b", "Teamcenter");
        simple!(r"(?i)\bmastercam\b", "Mastercam");
        simple!(r"(?i)\baltium\b", "Altium Designer");
        simple!(r"(?i)\borcad\b", "OrCAD");
        simple!(r"(?i)\bkicad\b", "KiCad");
        simple!(r"(?i)\brevit\b", "Revit");

        // Finance & Data
        simple!(r"(?i)\bbloomberg\b", "Bloomberg Terminal");
        simple!(r"(?i)\bfactset\b", "FactSet");
        simple!(r"(?i)\bcapitalline\b", "CapitalLine");
        simple!(r"(?i)\bmorningstar\b", "Morningstar");
        strict_dist!(r"(?i)\bstata\b", "STATA", r"(?i)\b(statistical|data|analysis|research|quantitative|survey|econometrics)\b", 15);
        strict_dist!(r"(?i)\bsas\b", "SAS", r"(?i)\b(statistical|programming|data|analytics|business|intelligence|software)\b", 15);

        // FinTech specifics
        simple!(r"(?i)\breuters eikon\b", "Reuters Eikon");
        simple!(r"(?i)\bquickbooks\b", "QuickBooks");
        simple!(r"(?i)\bxero\b", "Xero");
        simple!(r"(?i)\bsage (intacct|50|100|200|300|erp)\b", "Sage");
        simple!(r"(?i)\bintacct\b", "Intacct");
        simple!(r"(?i)\bstripe\b", "Stripe");
        simple!(r"(?i)\badyen\b", "Adyen");
        simple!(r"(?i)\bplaid\b", "Plaid");
        simple!(r"(?i)\bsquare\b", "Square");

        simple!(r"(?i)\bblockchain\b", "Blockchain");
        simple!(r"(?i)\bsolidity\b", "Solidity");
        simple!(r"(?i)\bsmart contracts\b", "Smart Contracts");
        simple!(r"(?i)\bethereum\b", "Ethereum");
        simple!(r"(?i)\bbitcoin\b", "Bitcoin");
        simple!(r"(?i)\bdefi\b|decentralized finance\b", "DeFi");
        simple!(r"(?i)\bnft\b", "NFT");

        // Operations & General Jargon
        strict_dist!(r"(?i)\bagile\b", "Agile", r"(?i)\b(scrum|kanban|methodology|environment|team|workflow|sprint|coach|practice|principles)\b", 15);
        simple!(r"(?i)\bscrum\b", "Scrum");
        simple!(r"(?i)\bkanban\b", "Kanban");
        strict_dist!(r"(?i)\blean\b", "Lean", r"(?i)\b(manufacturing|six sigma|process|production|principles|management|improvement|startup)\b", 15);
        simple!(r"(?i)\bsix sigma\b", "Six Sigma");
        simple!(r"(?i)\bproject management professional\b|\bpmp\b", "PMP");
        strict_dist!(r"(?i)\bpr\b", "Public Relations", r"(?i)\b(relations|media|communications|campaign|press|outreach|social|strategy)\b", 15);
        simple!(r"(?i)\bcopywriting\b", "Copywriting");
        simple!(r"(?i)\btechnical writing\b", "Technical Writing");
        simple!(r"(?i)\bgrant writing\b", "Grant Writing");
        simple!(r"(?i)\bcorporate social responsibility\b|\bcsr\b", "CSR");
        simple!(r"(?i)\besg\b|environmental social governance\b", "ESG");
        simple!(r"(?i)\bcustomer success\b", "Customer Success");
        strict_dist!(r"(?i)\bsaas\b", "SaaS", r"(?i)\b(software|platform|cloud|delivery|product|business|model|sales)\b", 15);
        simple!(r"(?i)\bpaas\b|platform as a service\b", "PaaS");
        simple!(r"(?i)\biaas\b|infrastructure as a service\b", "IaaS");
        simple!(r"(?i)\bfinops\b", "FinOps");
        simple!(r"(?i)\brevops\b", "RevOps");
        simple!(r"(?i)\bmarkops\b", "MarkOps");
        simple!(r"(?i)\bsalesops\b", "SalesOps");
        
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
        
        // LegalTech specifics
        simple!(r"(?i)\blexisnexis\b|\blexis nexis\b", "LexisNexis");
        simple!(r"(?i)\bwestlaw\b", "Westlaw");
        simple!(r"(?i)\brelativity\b", "Relativity");
        simple!(r"(?i)\bclio\b", "Clio");
        simple!(r"(?i)\beverlaw\b", "Everlaw");
        simple!(r"(?i)\bimanage\b", "iManage");
        simple!(r"(?i)\bnetdocuments\b", "NetDocuments");
        simple!(r"(?i)\bironclad\b", "Ironclad");
        simple!(r"(?i)\bbloomberg law\b", "Bloomberg Law");

        // Security & Cybersecurity specifics
        simple!(r"(?i)\bburp suite\b", "Burp Suite");
        simple!(r"(?i)\bmetasploit\b", "Metasploit");
        simple!(r"(?i)\bwireshark\b", "Wireshark");
        simple!(r"(?i)\bsplunk\b", "Splunk");
        simple!(r"(?i)\bnessus\b", "Nessus");
        simple!(r"(?i)\bokta\b", "Okta");
        simple!(r"(?i)\bcrowdstrike\b", "CrowdStrike");
        simple!(r"(?i)\bsentinelone\b", "SentinelOne");

        // HR & Recruiter Tech specifics
        simple!(r"(?i)\bgreenhouse\b", "Greenhouse");
        simple!(r"(?i)\blever\b", "Lever");
        simple!(r"(?i)\bashby\b", "Ashby");
        simple!(r"(?i)\bbamboohr\b", "BambooHR");
        simple!(r"(?i)\brippling\b", "Rippling");

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
        degree!(r"\b(md|jd|llb|llm|dds|dvm)\b", "Professional Degree");

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
        subject!(r"\b(civil engineering)\b", "Civil Engineering");
        subject!(r"\b(chemical engineering)\b", "Chemical Engineering");
        subject!(r"\b(biomedical engineering)\b", "Biomedical Engineering");
        subject!(r"\b(aerospace engineering)\b", "Aerospace Engineering");
        subject!(r"\b(industrial engineering)\b", "Industrial Engineering");
        subject!(r"\b(engineering)\b", "Engineering");
        
        // Science
        subject!(r"\bphysics\b", "Physics");
        subject!(r"\bchemistry\b", "Chemistry");
        subject!(r"\b(biology|biological sciences)\b", "Biology");
        subject!(r"\b(biochemistry|molecular biology)\b", "Biochemistry");
        subject!(r"\b(biotechnology|biotech)\b", "Biotechnology");
        subject!(r"\b(environmental science|ecology)\b", "Environmental Science");
        subject!(r"\b(geology|earth science)\b", "Geology");
        subject!(r"\b(psychology|behavioral science)\b", "Psychology");
        subject!(r"\b(neuroscience)\b", "Neuroscience");

        // Social Sciences & Humanities
        subject!(r"\b(economics|political economy)\b", "Economics");
        subject!(r"\b(political science|government|politics)\b", "Political Science");
        subject!(r"\b(sociology)\b", "Sociology");
        subject!(r"\b(anthropology)\b", "Anthropology");
        subject!(r"\b(international relations|global affairs)\b", "International Relations");
        subject!(r"\b(history)\b", "History");
        subject!(r"\b(philosophy)\b", "Philosophy");
        subject!(r"\b(english|literature|creative writing)\b", "English");
        subject!(r"\b(communications|media studies|journalism)\b", "Communications");
        subject!(r"\b(linguistics)\b", "Linguistics");
        subject!(r"\b(arts?|fine arts|visual arts|art history)\b", "Arts");
        subject!(r"\b(music|musicology)\b", "Music");
        
        // Professional & Other (Restored)
        subject!(r"\b(architecture)\b", "Architecture");
        subject!(r"\b(law|legal studies|jurisprudence)\b", "Law");
        subject!(r"\b(education|teaching|pedagogy)\b", "Education");
        subject!(r"\b(nursing)\b", "Nursing");
        subject!(r"\b(healthcare administration|public health)\b", "Healthcare");
        subject!(r"\b(medicine|medical studies)\b", "Medicine");
        subject!(r"\b(pharmacy|pharmaceutical sciences)\b", "Pharmacy");
        subject!(r"\b(dentistry|dental medicine)\b", "Dentistry");
        subject!(r"\b(veterinary medicine|vet science)\b", "Veterinary Medicine");
        subject!(r"\b(social work)\b", "Social Work");


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

    #[test]
    fn test_telehealth_tags() {
        let engine = TagEngine::new();
        let text = "Seeking a developer for our telehealth platform. Experience with Epic, Cerner, and HL7/FHIR is required. Knowledge of HIPAA compliance is a must.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("Telehealth"));
        assert!(tags_set.contains("Epic Systems"));
        assert!(tags_set.contains("Cerner"));
        assert!(tags_set.contains("HL7"));
        assert!(tags_set.contains("FHIR"));
        assert!(tags_set.contains("HIPAA Compliance"));
    }

    #[test]
    fn test_business_tech_tags() {
        let engine = TagEngine::new();
        let text = "We use HubSpot for marketing, Zendesk for support, and Jira/Confluence for project management. Experience with SAP or Oracle ERP is a plus.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("HubSpot"));
        assert!(tags_set.contains("Zendesk"));
        assert!(tags_set.contains("Jira"));
        assert!(tags_set.contains("Confluence"));
        assert!(tags_set.contains("SAP"));
        assert!(tags_set.contains("Oracle ERP"));
    }

    #[test]
    fn test_new_languages() {
        let engine = TagEngine::new();
        assert!(engine.detect_tags("Expert in Haskell and Erlang").contains(&"Haskell"));
        assert!(engine.detect_tags("Lisp or Clojure experience").contains(&"Clojure"));
    }

    #[test]
    fn test_business_tools() {
        let engine = TagEngine::new();
        assert!(engine.detect_tags("Using Google Workspace and MS Excel").contains(&"Google Workspace"));
        assert!(engine.detect_tags("Microsoft Word and Powerpoint proficiency").contains(&"Microsoft Office"));
        assert!(engine.detect_tags("Managing ERP systems").contains(&"ERP"));
    }

    #[test]
    fn test_specialized_field_tools() {
        let engine = TagEngine::new();
        
        // LegalTech
        let legal = engine.detect_tags("Familiar with LexisNexis, Westlaw, and Relativity");
        assert!(legal.contains(&"LexisNexis"));
        assert!(legal.contains(&"Westlaw"));
        assert!(legal.contains(&"Relativity"));

        // HealthTech
        let health = engine.detect_tags("Experience with Athenahealth or Meditech");
        assert!(health.contains(&"Athenahealth"));
        assert!(health.contains(&"Meditech"));

        // FinTech
        let finance = engine.detect_tags("Proficiency in QuickBooks and Xero");
        assert!(finance.contains(&"QuickBooks"));
        assert!(finance.contains(&"Xero"));

        // Engineering
        let eng = engine.detect_tags("Skills in Altium, Revit, and AutoCAD");
        assert!(eng.contains(&"Altium Designer"));
        assert!(eng.contains(&"Revit"));
        assert!(eng.contains(&"AutoCAD"));
    }

    #[test]
    fn test_new_education_subjects() {
        let detector = EducationDetector::new();
        
        let med = detector.detect("Student studying Medicine");
        assert!(med.subject_areas.contains(&"Medicine".to_string()));

        let pharm = detector.detect("Pursuing a degree in Pharmaceutical Sciences");
        assert!(pharm.subject_areas.contains(&"Pharmacy".to_string()));

        let dent = detector.detect("Enrolled in Dentistry school");
        assert!(dent.subject_areas.contains(&"Dentistry".to_string()));

        let vet = detector.detect("Currently in Vet Science program");
        assert!(vet.subject_areas.contains(&"Veterinary Medicine".to_string()));

        let nursing = detector.detect("Nursing student graduating soon");
        assert!(nursing.subject_areas.contains(&"Nursing".to_string()));
    }

    #[test]
    fn test_professional_degrees() {
        let detector = EducationDetector::new();
        
        let jd = detector.detect("JD candidate 2026");
        assert!(jd.degree_levels.contains(&"Professional Degree".to_string()));

        let md = detector.detect("MD student in clinical rotations");
        assert!(md.degree_levels.contains(&"Professional Degree".to_string()));

        let llm = detector.detect("Pursuing an LLM degree");
        assert!(llm.degree_levels.contains(&"Professional Degree".to_string()));
    }

    #[test]
    fn test_engineering_science_tags() {
        let engine = TagEngine::new();
        let text = "Position requires experience with Robotics, ROS, and CAD (SolidWorks/AutoCAD). Familiarity with MATLAB and FPGA (Verilog/VHDL) is desired.";
        let tags = engine.detect_tags(text);
        let tags_set: HashSet<_> = tags.iter().cloned().collect();

        assert!(tags_set.contains("Robotics"));
        assert!(tags_set.contains("ROS"));
        assert!(tags_set.contains("CAD"));
        assert!(tags_set.contains("SolidWorks"));
        assert!(tags_set.contains("AutoCAD"));
        assert!(tags_set.contains("MATLAB"));
        assert!(tags_set.contains("FPGA"));
        assert!(tags_set.contains("Verilog"));
        assert!(tags_set.contains("VHDL"));
    }

    #[test]
    fn test_expanded_education_subjects() {
        let detector = EducationDetector::new();
        
        // Physics and Chemistry
        let info = detector.detect("Student pursuing a degree in Physics and Chemistry");
        assert!(info.subject_areas.contains(&"Physics".to_string()));
        assert!(info.subject_areas.contains(&"Chemistry".to_string()));

        // Psychology and Sociology
        let info = detector.detect("Candidate studying Psychology or Sociology");
        assert!(info.subject_areas.contains(&"Psychology".to_string()));
        assert!(info.subject_areas.contains(&"Sociology".to_string()));

        // Architecture and Law
        let info = detector.detect("Enrolled in Architecture or Law studies");
        assert!(info.subject_areas.contains(&"Architecture".to_string()));
        assert!(info.subject_areas.contains(&"Law".to_string()));
    }

    #[test]
    fn test_strict_new_rules() {
        let engine = TagEngine::new();
        
        // Snowflake
        assert!(engine.detect_tags("Experience with Snowflake data warehouse").contains(&"Snowflake"));
        assert!(!engine.detect_tags("I found a beautiful snowflake").contains(&"Snowflake"));

        // Epic
        assert!(engine.detect_tags("Epic Systems EHR certification").contains(&"Epic Systems"));
        assert!(!engine.detect_tags("That was an epic fail").contains(&"Epic Systems"));

        // Unity
        assert!(engine.detect_tags("Unity game engine developer").contains(&"Unity"));
        assert!(!engine.detect_tags("Call for national unity").contains(&"Unity"));

        // CAD
        assert!(engine.detect_tags("Proficient in CAD software").contains(&"CAD"));
        assert!(!engine.detect_tags("The cad was very rude").contains(&"CAD"));

        // Agile
        assert!(engine.detect_tags("Working in an Agile scrum environment").contains(&"Agile"));
        assert!(!engine.detect_tags("He is very agile on his feet").contains(&"Agile"));
    }
}
