import { Hono } from 'hono'
import { renderer } from './renderer'
import type { Job } from './types'

type Env = {
  Bindings: {
    DB: D1Database
  }
}

type SearchParams = {
  query?: string
  location?: string
  tag?: string
  source?: string
  posted?: string
  page?: number
}

const app = new Hono<Env>()

app.use(renderer)

// Helper functions

const getTagStyle = (name: string) => {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  // Hue is determined by hash (0-360)
  // Saturation: 70% (Pastel but vibrant)
  // Lightness: 96% (Background), 25% (Text), 85% (Border)
  const h = Math.abs(hash % 360);
  return `background-color: hsl(${h}, 70%, 96%); color: hsl(${h}, 80%, 25%); border: 1px solid hsl(${h}, 60%, 85%);`
}

const formatDate = (dateString: string) => {
  if (!dateString) return ''
  try {
    const date = new Date(dateString)
    return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).format(date)
  } catch (e) {
    return dateString
  }
}

const formatAts = (ats: string) => {
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

const JobCard = ({ job }: { job: Job }) => {
  const iconLetter = job.company ? job.company.charAt(0) : '?'

  return (
    <div class="job-card" data-job-id={job.id}>
      <div class="card-header">
        <div class="company-info">
          <div class="company-icon">{iconLetter}</div>
          <div class="company-name">{job.company}</div>
        </div>
        <div class="header-right">
          {job.posted && (
            <div class="posted-date">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
        <h3 class="job-title">{job.title}</h3>
        <div class="location-row">
          {job.location && job.location.split(';').map((loc: string) => loc.trim()).filter(Boolean).map((loc: string) => (
            <div class="dashed-tag">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path>
                <circle cx="12" cy="10" r="3"></circle>
              </svg>
              {formatLocation(loc)}
            </div>
          ))}
          {job.departments && job.departments.map((dept: string) => (
            <span class="dashed-tag">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
              {dept}
            </span>
          ))}
          {job.tags?.slice(0, 7).map((tag: string) => (
            <span class="tag" style={getTagStyle(tag)}>{tag}</span>
          ))}
          {(job.tags?.length || 0) > 7 && (
            <span class="tag tag-more">+{job.tags!.length - 7}</span>
          )}
        </div>
      </div>

      <div class="card-footer">
        <div class="tags">
          <span class="tag ats-tag" style={getTagStyle(job.ats)}>{formatAts(job.ats)}</span>
        </div>
        <a href={job.url} target="_blank" class="apply-btn">Apply</a>
      </div>
    </div>
  )
}

const getSearchParams = (c: any): SearchParams => {
  return {
    query: c.req.query('q'),
    location: c.req.query('location'),
    tag: c.req.query('tag'),
    source: c.req.query('source'),
    posted: c.req.query('posted'),
    page: parseInt(c.req.query('page') || '1'),
  }
}

const getJobs = async (
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
      const conditions = locations.map(() => 'location LIKE ? ESCAPE "\\"').join(' OR ')
      whereClause += ` AND (${conditions})`
      locations.forEach(loc => {
        sqlParams.push(`%${escapeLike(loc)}%`)
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

  // Count query
  const countSql = `SELECT COUNT(*) as total FROM jobs ${whereClause}`

  // Company count query
  const companyCountSql = `SELECT COUNT(DISTINCT company) as total FROM jobs ${whereClause}`

  // Data query
  const dataSql = `SELECT * FROM jobs ${whereClause} ORDER BY created_at DESC LIMIT ? OFFSET ?`
  const dataParams = [...sqlParams, limit, offset]

  try {
    // Get total count
    const { results: countResults } = await db.prepare(countSql).bind(...sqlParams).all()
    const total = countResults[0].total as number

    // Get unique companies
    const { results: companyResults } = await db.prepare(companyCountSql).bind(...sqlParams).all()
    const companyCount = companyResults[0].total as number

    // Get jobs
    const { results } = await db.prepare(dataSql).bind(...dataParams).all()
    const jobs = results as unknown as Job[]

    if (jobs.length > 0) {
      const jobIds = jobs.map(j => j.id)
      const placeholders = jobIds.map(() => '?').join(',')

      // Batch fetch tags
      const { results: allTags } = await db.prepare(
        `SELECT job_id, name FROM job_tags WHERE job_id IN (${placeholders})`
      ).bind(...jobIds).all()

      // Batch fetch departments
      const { results: allDepts } = await db.prepare(
        `SELECT job_id, name FROM job_departments WHERE job_id IN (${placeholders})`
      ).bind(...jobIds).all()

      // Map results back to jobs
      const tagMap = (allTags as any[]).reduce((acc, t) => {
        acc[t.job_id] = acc[t.job_id] || []
        acc[t.job_id].push(t.name)
        return acc
      }, {} as Record<string, string[]>)

      const deptMap = (allDepts as any[]).reduce((acc, d) => {
        acc[d.job_id] = acc[d.job_id] || []
        acc[d.job_id].push(d.name)
        return acc
      }, {} as Record<string, string[]>)

      for (const job of jobs) {
        job.tags = tagMap[job.id] || []
        job.departments = deptMap[job.id] || []
      }
    }

    const latency = Math.round(performance.now() - start)
    return { jobs, total, companyCount, latency }
  } catch (e) {
    console.error('D1 Error:', e)
    return { jobs: [], total: 0, companyCount: 0, latency: 0 }
  }
}

// --- Components ---

const ThemeToggle = () => (
  <button id="themeToggle" class="theme-toggle" title="Toggle Theme" aria-label="Toggle Theme">
    <svg class="sun-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
    <svg class="moon-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
    </svg>
  </button>
)

const SearchFilters = ({ params, total, companyCount, latency }: { params: SearchParams; total: number; companyCount: number; latency: number }) => {
  const { query, location, tag, source, posted } = params
  const isFilterVisible = location || tag || source || posted

  return (
    <div class="search-container">
      <form method="get" action="/" class="search-form" id="searchForm">
        <div class="search-row">
          <div class="search-input-wrapper">
            <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input
              type="text"
              name="q"
              class="search-input"
              placeholder="Search roles or companies..."
              value={query || ''}
              autocomplete="off"
            />
          </div>

          <button type="button" class="btn-filter" id="filterToggle" title="Toggle Filters">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
          <input type="hidden" name="source" id="sourceInput" value={source || ''} />

          <div class="filter-group">
            <label class="filter-label">Location</label>
            <div class="tag-input-container" id="locationTagContainer">
              <div class="tag-pills" id="locationPills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. London, Remote" autocomplete="off" />
            </div>
          </div>

          <div class="filter-group">
            <label class="filter-label">Tags</label>
            <div class="tag-input-container" id="tagTagContainer">
              <div class="tag-pills" id="tagPills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. Python, React" autocomplete="off" />
            </div>
          </div>

          <div class="filter-group">
            <label class="filter-label">Source</label>
            <div class="tag-input-container" id="sourceTagContainer">
              <div class="tag-pills" id="sourcePills"></div>
              <input type="text" class="tag-input-field" placeholder="e.g. Greenhouse, Ashby" autocomplete="off" />
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
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
      <h3>Select a role</h3>
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

  return c.render(
    <>
      <ThemeToggle />
      <header>
        <div class="header-title">
          <h1>Explore Roles</h1>
          <p>Early career search engine for students.</p>
        </div>
      </header>

      <SearchFilters params={params} total={total} companyCount={companyCount} latency={latency} />

      <div class="content-wrapper">
        <div class="main-content">
          <div class="jobs-grid">
            {jobs.length > 0 ? (
              jobs.map((job) => <JobCard job={job} />)
            ) : (
              <div class="no-results">No results found.</div>
            )}
          </div>
        </div>
        <DetailPanel />
      </div>
      <div class="panel-overlay" id="panelOverlay"></div>
    </>
  )
})

app.get('/api/jobs', async (c) => {
  const params = getSearchParams(c)
  const { jobs } = await getJobs(c.env.DB, params)

  if (jobs.length === 0) {
    return c.body(null, 204) // No content
  }

  return c.html(
    <>
      {jobs.map((job) => <JobCard job={job} />)}
    </>
  )
})

app.get('/api/job/:id', async (c) => {
  const id = c.req.param('id')

  try {
    const { results } = await c.env.DB.prepare(
      'SELECT * FROM jobs WHERE id = ?'
    ).bind(id).all()

    if (results.length === 0) {
      return c.json({ error: 'Job not found' }, 404)
    }

    const job = results[0] as any

    // Fetch tags
    const { results: tags } = await c.env.DB.prepare(
      'SELECT name FROM job_tags WHERE job_id = ?'
    ).bind(id).all()
    job.tags = tags.map((t: any) => t.name)

    // Fetch departments
    const { results: depts } = await c.env.DB.prepare(
      'SELECT name FROM job_departments WHERE job_id = ?'
    ).bind(id).all()
    job.departments = depts.map((d: any) => d.name)

    return c.json(job)
  } catch (e) {
    console.error('D1 Error:', e)
    return c.json({ error: 'Database error' }, 500)
  }
})

export default app
