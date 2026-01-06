-- Database schema for Zapply jobs
DROP TABLE IF EXISTS job_tags;
DROP TABLE IF EXISTS job_offices;
DROP TABLE IF EXISTS job_departments;
DROP TABLE IF EXISTS jobs;

CREATE TABLE IF NOT EXISTS industries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    company TEXT NOT NULL,
    slug TEXT NOT NULL,
    ats TEXT NOT NULL,
    url TEXT NOT NULL,
    location TEXT,
    posted TEXT,
    industry_id INTEGER REFERENCES industries(id),
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

CREATE INDEX IF NOT EXISTS idx_jobs_company ON jobs(company);
CREATE INDEX IF NOT EXISTS idx_jobs_posted ON jobs(posted);
CREATE INDEX IF NOT EXISTS idx_job_departments_job_id ON job_departments(job_id);
CREATE INDEX IF NOT EXISTS idx_job_offices_job_id ON job_offices(job_id);
CREATE INDEX IF NOT EXISTS idx_job_tags_job_id ON job_tags(job_id);
CREATE INDEX IF NOT EXISTS idx_jobs_industry_id ON jobs(industry_id);
