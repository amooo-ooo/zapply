CREATE TABLE IF NOT EXISTS countries (
    code TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS regions (
    id TEXT PRIMARY KEY, -- e.g. "US.CA"
    country_code TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (country_code) REFERENCES countries(code) ON DELETE CASCADE
);

DROP TABLE IF EXISTS job_tags;
DROP TABLE IF EXISTS job_offices;
DROP TABLE IF EXISTS job_departments;
DROP TABLE IF EXISTS jobs;

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    company TEXT NOT NULL,
    slug TEXT NOT NULL,
    ats TEXT NOT NULL,
    url TEXT NOT NULL,
    company_url TEXT,
    location TEXT,
    city TEXT,
    region TEXT,
    country TEXT,
    country_code TEXT,
    posted TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS job_departments (
    job_id TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, name)
);

CREATE TABLE IF NOT EXISTS job_offices (
    job_id TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, name)
);

CREATE TABLE IF NOT EXISTS job_tags (
    job_id TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, name)
);

CREATE TABLE IF NOT EXISTS job_degree_levels (
    job_id TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, name)
);

CREATE TABLE IF NOT EXISTS job_subject_areas (
    job_id TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, name)
);

CREATE INDEX IF NOT EXISTS idx_jobs_company ON jobs(company);
CREATE INDEX IF NOT EXISTS idx_jobs_posted ON jobs(posted);
CREATE INDEX IF NOT EXISTS idx_jobs_title ON jobs(title);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at_desc ON jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_city ON jobs(city);
CREATE INDEX IF NOT EXISTS idx_jobs_region ON jobs(region);
CREATE INDEX IF NOT EXISTS idx_jobs_country ON jobs(country);
CREATE INDEX IF NOT EXISTS idx_jobs_country_code ON jobs(country_code);

CREATE INDEX IF NOT EXISTS idx_countries_name ON countries(name);

CREATE INDEX IF NOT EXISTS idx_regions_country_code ON regions(country_code);
CREATE INDEX IF NOT EXISTS idx_regions_name ON regions(name);

CREATE INDEX IF NOT EXISTS idx_job_departments_job_id ON job_departments(job_id);
CREATE INDEX IF NOT EXISTS idx_job_departments_name ON job_departments(name);

CREATE INDEX IF NOT EXISTS idx_job_offices_job_id ON job_offices(job_id);
CREATE INDEX IF NOT EXISTS idx_job_offices_name ON job_offices(name);

CREATE INDEX IF NOT EXISTS idx_job_tags_job_id ON job_tags(job_id);
CREATE INDEX IF NOT EXISTS idx_job_tags_name ON job_tags(name);

CREATE INDEX IF NOT EXISTS idx_job_degree_levels_job_id ON job_degree_levels(job_id);
CREATE INDEX IF NOT EXISTS idx_job_degree_levels_name ON job_degree_levels(name);

CREATE INDEX IF NOT EXISTS idx_job_subject_areas_job_id ON job_subject_areas(job_id);
CREATE INDEX IF NOT EXISTS idx_job_subject_areas_name ON job_subject_areas(name);
