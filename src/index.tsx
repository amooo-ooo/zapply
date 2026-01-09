import { Hono } from 'hono'
import { renderer } from './renderer'
import type { Job } from './types'

import { cache } from 'hono/cache'

export type Env = {
  Bindings: {
    DB: D1Database
    LOGO_DEV_TOKEN: string
  }
}

export type SearchParams = {
  query?: string
  location?: string
  tag?: string
  company?: string
  source?: string
  degree?: string
  field?: string
  posted?: string
  page?: number
}

const app = new Hono<Env>()

app.use('*', cache({
  cacheName: 'zapply-cache',
  cacheControl: 'max-age=60',
}))

app.use(renderer)

import api from './routes/api'
app.route('/api', api)

// Helper functions

export const getTagStyle = (name: string) => {
  const hues = [
    217, // Blue
    142, // Green
    273, // Purple
    38,  // Amber
    350, // Rose
    189, // Cyan
    239, // Indigo
    14,  // Orange
  ]

  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }

  const h = hues[Math.abs(hash) % hues.length];
  return `--tag-hue: ${h};`
}

export const formatDate = (dateString: string) => {
  if (!dateString) return ''
  try {
    const date = new Date(dateString)
    return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).format(date)
  } catch (e) {
    return dateString
  }
}

export const formatAts = (ats: string) => {
  if (!ats) return ''
  return ats.replace(/['"]+/g, '').replace(/\b\w/g, (l) => l.toUpperCase())
}

const formatLocation = (loc: string) => {
  const parts = loc.trim().split(' ')
  if (parts.length > 1) {
    const lastPart = parts[parts.length - 1]
    if (lastPart.length === 2 && /^[a-zA-Z]{2}$/.test(lastPart)) {
      parts[parts.length - 1] = lastPart.toUpperCase()
      return parts.join(' ')
    }
  }
  return loc.trim()
}



export const JobCard = ({ job, token }: { job: Job; token: string }) => {
  const iconLetter = job.company ? job.company.charAt(0) : '?'

  // Resolve logo query dynamically
  let logoQuery = job.company.replace(/-/g, '')
  if (job.company_url) {
    try {
      const urlStr = job.company_url.startsWith('http') ? job.company_url : `https://${job.company_url}`
      logoQuery = new URL(urlStr).hostname
    } catch (e) {
      // Use fallback (dash-removed company name)
    }
  }

  const logoUrl = `https://img.logo.dev/${encodeURIComponent(logoQuery)}?token=${token}`

  return (
    <article class="job-card" data-job-id={job.id} aria-labelledby={`job-title-${job.id}`} tabIndex={0}>
      <div class="card-header">
        <div class="company-info">
          <div class="company-icon">
            <img
              src={logoUrl}
              alt={job.company}
              loading="lazy"
              width="32"
              height="32"
              style="display: block; width: 100%; height: 100%; object-fit: contain; border-radius: 6px;"
              onerror="this.style.display='none'; this.nextElementSibling.style.display='flex'"
            />
            <span style="display: none; width: 100%; height: 100%; align-items: center; justify-content: center;">{iconLetter}</span>
          </div>
          <div class="company-name">
            {job.company_url ? (
              <a
                href={job.company_url.startsWith('http') ? job.company_url : `https://${job.company_url}`}
                target="_blank"
                rel="noopener noreferrer"
                style="text-decoration: none; color: inherit;"
              >
                {job.company}
              </a>
            ) : (
              job.company
            )}
          </div>
        </div>
        <div class="header-right">
          {job.posted && (
            <div class="posted-date">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="16" y1="2" x2="16" y2="6"></line>
                <line x1="8" y1="2" x2="8" y2="6"></line>
                <line x1="3" y1="10" x2="21" y2="10"></line>
              </svg>
              {formatDate(job.posted)}
            </div>
          )}
        </div>
      </div>

      <div class="card-body">
        <h2 class="job-title" id={`job-title-${job.id}`}>{job.title}</h2>

        <div class="job-metadata">
          {job.location && (
            <div class="metadata-item">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path>
                <circle cx="12" cy="10" r="3"></circle>
              </svg>
              <span style="display: flex; flex-direction: column;">
                {job.city || job.region || job.country ? (
                  [job.city, job.region, job.country].filter(Boolean).join(', ')
                ) : (
                  job.location.split(';').map((loc: string) => (
                    <span key={loc}>{formatLocation(loc.trim())}</span>
                  ))
                )}
              </span>
            </div>
          )}

          {job.degree_levels && job.degree_levels.length > 0 && (
            <div class="metadata-item">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M22 10v6M2 10l10-5 10 5-10 5z"></path>
                <path d="M6 12v5c3 3 9 3 12 0v-5"></path>
              </svg>
              <span>{job.degree_levels.join(', ')}</span>
            </div>
          )}

          {job.subject_areas && job.subject_areas.length > 0 && (
            <div class="metadata-item">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
              </svg>
              <span>{job.subject_areas.join(', ')}</span>
            </div>
          )}
        </div>

        <div class="tag-row">
          {job.tags?.slice(0, 10).map((tag: string) => {
            const isRainbow = tag.toUpperCase().includes('LGBTQ')
            return (
              <span
                class={`tag ${isRainbow ? 'tag-rainbow' : ''}`}
                style={isRainbow ? '' : getTagStyle(tag)}
              >
                {tag}
              </span>
            )
          })}
          {(job.tags?.length || 0) > 10 && (
            <span class="tag tag-more">+{job.tags!.length - 10}</span>
          )}
        </div>
      </div>

      <div class="card-footer">
        <div class="tags">
          <span class="tag ats-tag" style={getTagStyle(job.ats)}>{formatAts(job.ats)}</span>
        </div>
        <a href={job.url} target="_blank" class="apply-btn" aria-label={`Apply for ${job.title} at ${job.company}`}>Apply</a>
      </div>
    </article>
  )
}
export const getSearchParams = (c: any): SearchParams => {
  return {
    query: c.req.query('q'),
    location: c.req.query('location'),
    tag: c.req.query('tag'),
    company: c.req.query('company'),
    source: c.req.query('source'),
    degree: c.req.query('degree'),
    field: c.req.query('field'),
    posted: c.req.query('posted'),
    page: parseInt(c.req.query('page') || '1'),
  }
}

export const getJobs = async (
  db: D1Database,
  params: SearchParams
): Promise<{ jobs: Job[]; total: number; companyCount: number; latency: number }> => {
  const start = performance.now()
  const page = params.page || 1
  const limit = 50
  const offset = (page - 1) * limit

  const escapeLike = (str: string) => {
    return str.replace(/[\\%_]/g, '\\$&')
  }

  let whereClause = 'WHERE 1=1'
  let sqlParams: any[] = []

  if (params.query) {
    const escapedQuery = escapeLike(params.query)
    whereClause += ' AND (title LIKE ? ESCAPE "\\" OR company LIKE ? ESCAPE "\\")'
    sqlParams.push(`%${escapedQuery}%`, `%${escapedQuery}%`)
  }

  if (params.location) {
    const locations = params.location.split(',').map(l => l.trim()).filter(Boolean)
    if (locations.length > 0) {
      const conditions = locations.map(() => '(location LIKE ? ESCAPE "\\" OR city LIKE ? ESCAPE "\\" OR region LIKE ? ESCAPE "\\" OR country LIKE ? ESCAPE "\\" OR country_code LIKE ? ESCAPE "\\")').join(' OR ')
      whereClause += ` AND (${conditions})`
      locations.forEach(loc => {
        const escaped = `%${escapeLike(loc)}%`
        sqlParams.push(escaped, escaped, escaped, escaped, escaped)
      })
    }
  }

  if (params.tag) {
    const tags = params.tag.split(',').map(t => t.trim()).filter(Boolean)
    if (tags.length > 0) {
      const conditions = tags.map(() => 'name LIKE ? ESCAPE "\\"').join(' OR ')
      whereClause += ` AND id IN (SELECT job_id FROM job_tags WHERE ${conditions})`
      tags.forEach(tag => {
        sqlParams.push(`%${escapeLike(tag)}%`)
      })
    }
  }

  if (params.company) {
    const companies = params.company.split(',').map(c => c.trim()).filter(Boolean)
    if (companies.length > 0) {
      const conditions = companies.map(() => 'company LIKE ? ESCAPE "\\"').join(' OR ')
      whereClause += ` AND (${conditions})`
      companies.forEach(company => {
        sqlParams.push(`%${escapeLike(company)}%`)
      })
    }
  }

  if (params.source) {
    const sources = params.source.split(',').map(s => s.trim().toLowerCase()).filter(Boolean)
    if (sources.length > 0) {
      const conditions = sources.map(() => 'ats = ?').join(' OR ')
      whereClause += ` AND (${conditions})`
      sources.forEach(source => {
        // Wrap in quotes because they are stored as JSON strings in D1 by the Rust scraper
        sqlParams.push(`"${source}"`)
      })
    }
  }

  if (params.posted) {
    let days = 0
    switch (params.posted) {
      case '24h': days = 1; break
      case '3d': days = 3; break
      case '7d': days = 7; break
      case '30d': days = 30; break
    }
    if (days > 0) {
      whereClause += ` AND datetime(posted) >= datetime('now', '-${days} days')`
    }
  }

  if (params.degree) {
    whereClause += ' AND id IN (SELECT job_id FROM job_degree_levels WHERE name = ?)'
    sqlParams.push(params.degree)
  }

  if (params.field) {
    const fields = params.field.split(',').map(f => f.trim()).filter(Boolean)
    if (fields.length > 0) {
      const conditions = fields.map(() => 'name LIKE ? ESCAPE "\\"').join(' OR ')
      whereClause += ` AND id IN (SELECT job_id FROM job_subject_areas WHERE ${conditions})`
      fields.forEach(f => {
        sqlParams.push(`%${escapeLike(f)}%`)
      })
    }
  }

  // Consolidated Counts and Data query with JSON aggregation
  const countSql = `SELECT COUNT(*) as total FROM jobs ${whereClause}`
  const companyCountSql = `SELECT COUNT(DISTINCT company) as total FROM jobs ${whereClause}`
  const dataSql = `
    SELECT 
      j.id, j.title, j.company, j.slug, j.ats, j.url, j.company_url, j.location, j.posted, j.created_at,
      (SELECT json_group_array(name) FROM job_tags WHERE job_id = j.id) as tags,
      (SELECT json_group_array(name) FROM job_departments WHERE job_id = j.id) as departments,
      (SELECT json_group_array(name) FROM job_degree_levels WHERE job_id = j.id) as degree_levels,
      (SELECT json_group_array(name) FROM job_subject_areas WHERE job_id = j.id) as subject_areas
    FROM jobs j
    ${whereClause}
    ORDER BY j.created_at DESC 
    LIMIT ? OFFSET ?
  `

  try {
    // Execute all in a single batch
    const [countRes, companyRes, dataRes] = await db.batch([
      db.prepare(countSql).bind(...sqlParams),
      db.prepare(companyCountSql).bind(...sqlParams),
      db.prepare(dataSql).bind(...sqlParams, limit, offset)
    ])

    const total = (countRes.results[0] as any).total
    const companyCount = (companyRes.results[0] as any).total

    const jobs = dataRes.results.map((row: any) => ({
      ...row,
      tags: JSON.parse(row.tags || '[]'),
      departments: JSON.parse(row.departments || '[]'),
      degree_levels: JSON.parse(row.degree_levels || '[]'),
      subject_areas: JSON.parse(row.subject_areas || '[]'),
    })) as unknown as Job[]

    const latency = Math.round(performance.now() - start)
    return { jobs, total, companyCount, latency }
  } catch (e) {
    console.error('D1 Error:', e)
    return { jobs: [], total: 0, companyCount: 0, latency: 0 }
  }
}

// --- Components ---

const THEMES = {
  neutral: {
    name: 'Neutral',
    primary: '#18181b',
    accent: '#3b82f6',
    preview: '#3f3f46',
    base: 'zinc'
  },
  stone: {
    name: 'Stone',
    primary: '#1c1917',
    accent: '#65a30d',
    preview: '#44403c',
    base: 'stone'
  },
  slate: {
    name: 'Slate',
    primary: '#0f172a',
    accent: '#6366f1',
    preview: '#334155',
    base: 'slate'
  },
  rose: {
    name: 'Rose',
    primary: '#4c0519',
    accent: '#f43f5e',
    preview: '#be123c',
    base: 'rose'
  },
  indigo: {
    name: 'Indigo',
    primary: '#1e1b4b',
    accent: '#6366f1',
    preview: '#4338ca',
    base: 'indigo'
  },
  emerald: {
    name: 'Emerald',
    primary: '#064e3b',
    accent: '#10b981',
    preview: '#047857',
    base: 'emerald'
  },
}

const ThemeSelector = () => (
  <div class="theme-menu" id="themeMenu">
    <button id="paletteToggle" class="theme-toggle" title="Change Accent Color" aria-label="Change Accent Color">
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-palette-icon lucide-palette" aria-hidden="true"><path d="M12 22a1 1 0 0 1 0-20 10 9 0 0 1 10 9 5 5 0 0 1-5 5h-2.25a1.75 1.75 0 0 0-1.4 2.8l.3.4a1.75 1.75 0 0 1-1.4 2.8z" /><circle cx="13.5" cy="6.5" r=".5" fill="currentColor" /><circle cx="17.5" cy="10.5" r=".5" fill="currentColor" /><circle cx="6.5" cy="12.5" r=".5" fill="currentColor" /><circle cx="8.5" cy="7.5" r=".5" fill="currentColor" /></svg>
    </button>
    <div class="theme-options" id="themeOptions">
      {Object.entries(THEMES).map(([id, theme], index) => (
        <button
          class="theme-pill"
          data-theme={id}
          title={theme.name}
          style={`--dot-color: ${theme.preview}; --index: ${index};`}
        >
          <span class="theme-pill-dot"></span>
          <span class="theme-pill-name">{theme.name}</span>
        </button>
      ))}
    </div>
  </div>
)

const SettingsMenu = () => (
  <div class="settings-menu-container">
    <button id="settingsToggle" class="theme-toggle" title="Settings" aria-label="Settings">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path>
        <circle cx="12" cy="12" r="3"></circle>
      </svg>
    </button>
    <div class="settings-dropdown" id="settingsDropdown">
      <button id="themeToggle" class="theme-toggle" title="Toggle Light/Dark" aria-label="Toggle Theme">
        <svg class="sun-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="5"></circle>
          <line x1="12" y1="1" x2="12" y2="3"></line>
          <line x1="12" y1="21" x2="12" y2="23"></line>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
          <line x1="1" y1="12" x2="3" y2="12"></line>
          <line x1="21" y1="12" x2="23" y2="12"></line>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
        </svg>
        <svg class="moon-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
        </svg>
      </button>
      <ThemeSelector />
    </div>
  </div>
)

const SearchFilters = ({ params, total, companyCount, latency }: { params: SearchParams; total: number; companyCount: number; latency: number }) => {
  const { query, location, tag, company, source, posted, degree, field } = params
  const isFilterVisible = location || tag || company || source || posted || degree || field

  return (
    <div class="search-container">
      <form method="get" action="/" class="search-form" id="searchForm">
        <div class="search-row">
          <div class="search-input-wrapper">
            <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input
              type="text"
              name="q"
              class="search-input"
              placeholder="Search roles or companies..."
              aria-label="Search roles or companies"
              value={query || ''}
              autocomplete="off"
            />
          </div>

          <button type="button" class="btn-filter" id="filterToggle" title="Toggle Filters">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="4" y1="21" x2="4" y2="14"></line>
              <line x1="4" y1="10" x2="4" y2="3"></line>
              <line x1="12" y1="21" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12" y2="3"></line>
              <line x1="20" y1="21" x2="20" y2="16"></line>
              <line x1="20" y1="12" x2="20" y2="3"></line>
              <line x1="1" y1="14" x2="7" y2="14"></line>
              <line x1="9" y1="8" x2="15" y2="8"></line>
              <line x1="17" y1="16" x2="23" y2="16"></line>
            </svg>
            <span>Filter</span>
          </button>

          <button type="submit" class="btn-search">Search</button>
        </div>

        <div class="filter-row" id="filterRow" style={isFilterVisible ? '' : 'display: none;'}>
          <input type="hidden" name="location" id="locationInput" value={location || ''} />
          <input type="hidden" name="tag" id="tagInput" value={tag || ''} />
          <input type="hidden" name="company" id="companyInput" value={company || ''} />
          <input type="hidden" name="source" id="sourceInput" value={source || ''} />

          <div class="filter-group filter-group-full">
            <label class="filter-label">Tags</label>
            <div class="tag-input-container" id="tagTagContainer">
              <div class="tag-pills" id="tagPills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. Python, React" autocomplete="off" />
              <div class="autocomplete-dropdown" id="tagAutocomplete"></div>
            </div>
          </div>

          <div class="filter-group">
            <label class="filter-label">Degree</label>
            <select name="degree" class="filter-select">
              <option value="">Any Degree</option>
              <option value="Bachelor's" selected={degree === "Bachelor's"}>Bachelor's</option>
              <option value="Master's" selected={degree === "Master's"}>Master's</option>
              <option value="PhD" selected={degree === "PhD"}>PhD</option>
              <option value="Associate's" selected={degree === "Associate's"}>Associate's</option>
            </select>
          </div>

          <div class="filter-group">
            <label class="filter-label">Field of Study</label>
            <input type="hidden" name="field" id="fieldInput" value={field || ''} />
            <div class="tag-input-container" id="fieldTagContainer">
              <div class="tag-pills" id="fieldPills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. Computer Science" autocomplete="off" />
              <div class="autocomplete-dropdown" id="fieldAutocomplete"></div>
            </div>
          </div>

          <div class="filter-group">
            <label class="filter-label">Company</label>
            <div class="tag-input-container" id="companyTagContainer">
              <div class="tag-pills" id="companyPills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. Google, Microsoft" autocomplete="off" />
              <div class="autocomplete-dropdown" id="companyAutocomplete"></div>
            </div>
          </div>

          <div class="filter-group">
            <label class="filter-label">Location</label>
            <div class="tag-input-container" id="locationTagContainer">
              <div class="tag-pills" id="locationPills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. London, Remote" autocomplete="off" />
            </div>
          </div>

          <div class="filter-group">
            <label class="filter-label">Posted Date</label>
            <select name="posted" class="filter-select">
              <option value="">Any Time</option>
              <option value="24h" selected={posted === '24h'}>Past 24 Hours</option>
              <option value="3d" selected={posted === '3d'}>Past 3 Days</option>
              <option value="7d" selected={posted === '7d'}>Past Week</option>
              <option value="30d" selected={posted === '30d'}>Past Month</option>
            </select>
          </div>

          <div class="filter-group">
            <label class="filter-label">Source</label>
            <div class="tag-input-container" id="sourceTagContainer">
              <div class="tag-pills" id="sourcePills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. Greenhouse, Ashby" autocomplete="off" />
              <div class="autocomplete-dropdown" id="sourceAutocomplete"></div>
            </div>
          </div>
        </div>
      </form>
      <StatsPills total={total} companyCount={companyCount} latency={latency} />
    </div>
  )
}

const StatsPills = ({ total, companyCount, latency }: { total: number; companyCount: number; latency: number }) => (
  <div class="stats-container">
    <div class="stat-pill">{total} results</div>
    <div class="stat-pill">{companyCount} {companyCount === 1 ? 'company' : 'companies'}</div>
    <div class="stat-pill">{latency}ms</div>
  </div>
)

const DetailPanel = () => (
  <aside class="detail-panel" id="detailPanel">
    <button class="close-panel" id="closePanel" aria-label="Close Details">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
    <div class="panel-default" id="panelDefault">
      <div class="panel-default-icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
          <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
        </svg>
      </div>
      <h2>Select a role</h2>
      <p>Click on a job card to view details</p>
    </div>
    <div class="panel-content" id="panelContent" style="display: none;">
      <div class="panel-header">
        <div class="panel-company-icon" id="panelCompanyIcon">?</div>
        <div class="panel-header-info">
          <h2 id="panelJobTitle">Job Title</h2>
          <p id="panelCompanyName">Company Name</p>
        </div>
      </div>
      <div class="panel-meta">
        <div class="panel-meta-item" id="panelLocation">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path>
            <circle cx="12" cy="10" r="3"></circle>
          </svg>
          <span>Location</span>
        </div>
        <div class="panel-meta-item" id="panelDegree" style="display: none;">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 10v6M2 10l10-5 10 5-10 5z"></path><path d="M6 12v5c3 3 9 3 12 0v-5"></path></svg>
          <span>Degree</span>
        </div>
        <div class="panel-meta-item" id="panelField" style="display: none;">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path></svg>
          <span>Field</span>
        </div>
        <div class="panel-meta-item" id="panelDept" style="display: none;">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
          <span>Department</span>
        </div>
        <div class="panel-meta-item" id="panelPosted">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
            <line x1="16" y1="2" x2="16" y2="6"></line>
            <line x1="8" y1="2" x2="8" y2="6"></line>
            <line x1="3" y1="10" x2="21" y2="10"></line>
          </svg>
          <span>Posted date</span>
        </div>
      </div>
      <div class="panel-tags" id="panelTags"></div>
      <div class="panel-description" id="panelDescription">
        <p>Job description will appear here...</p>
      </div>
      <div class="panel-footer">
        <a href="#" target="_blank" class="panel-apply-btn" id="panelApplyBtn">Apply Now</a>
      </div>
    </div>
  </aside>
)

app.get('/', async (c) => {
  const params = getSearchParams(c)
  const { jobs, total, companyCount, latency } = await getJobs(c.env.DB, params)

  // @ts-ignore
  return c.render(
    <>
      <SettingsMenu />
      <header>
        <div class="header-title">
          <h1>Explore Roles</h1>
          <p>Internship & early career search engine for students.</p>
        </div>
      </header>

      <SearchFilters params={params} total={total} companyCount={companyCount} latency={latency} />

      <main class="content-wrapper">
        <div class="main-content">
          <div class="jobs-grid">
            {jobs.length > 0 ? (
              jobs.map((job) => <JobCard job={job} token={c.env.LOGO_DEV_TOKEN || ''} />)
            ) : (
              <div class="no-results">No results found.</div>
            )}
          </div>
        </div>
        <DetailPanel />
      </main>
      <div class="panel-overlay" id="panelOverlay"></div>
    </>,
    { logoDevToken: c.env.LOGO_DEV_TOKEN || '' }
  )
})

export default app
