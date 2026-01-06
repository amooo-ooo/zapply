import { Hono } from 'hono'
import { renderer } from './renderer'
import type { Job } from './types'

type Env = {
  Bindings: {
    DB: D1Database
  }
}

const app = new Hono<Env>()

app.use(renderer)

// Helper functions (moved outside component)
const getTagClass = (name: string) => {
  const hash = name.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  const classes = ['tag-blue', 'tag-green', 'tag-purple', 'tag-yellow', 'tag-red', 'tag-default']
  return classes[hash % classes.length]
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
          {job.tags?.map((tag: string) => (
            <span class={`tag ${getTagClass(tag)}`}>{tag}</span>
          ))}
        </div>
      </div>

      <div class="card-footer">
        <div class="tags">
          <span class={`tag ${getTagClass(job.ats)} ats-tag`}>{formatAts(job.ats)}</span>
        </div>
        <a href={job.url} target="_blank" class="apply-btn">Apply</a>
      </div>
    </div>
  )
}

const getJobs = async (
  db: D1Database,
  params: { query?: string; location?: string; tag?: string; page?: number }
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

    // Fetch tags and departments for display
    for (const job of jobs) {
      const { results: tags } = await db.prepare(
        'SELECT name FROM job_tags WHERE job_id = ?'
      ).bind(job.id).all()
      job.tags = tags.map((t: any) => t.name)

      const { results: depts } = await db.prepare(
        'SELECT name FROM job_departments WHERE job_id = ?'
      ).bind(job.id).all()
      job.departments = depts.map((d: any) => d.name)
    }

    const latency = Math.round(performance.now() - start)
    return { jobs, total, companyCount, latency }
  } catch (e) {
    console.error('D1 Error:', e)
    return { jobs: [], total: 0, companyCount: 0, latency: 0 }
  }
}

app.get('/', async (c) => {
  const query = c.req.query('q')
  const location = c.req.query('location')
  const tag = c.req.query('tag')

  if ((query && query.length > 100) || (location && location.length > 100) || (tag && tag.length > 100)) {
    return c.text('Input too long', 400)
  }

  const { jobs, total, companyCount, latency } = await getJobs(c.env.DB, { query, location, tag })

  return c.render(
    <>
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

      <header>
        <div class="header-title">
          <h1>Explore Roles</h1>
          <p>Early career search engine for students.</p>
        </div>
      </header>

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

            <button type="submit" class="btn-search">
              Search
            </button>
          </div>

          <div class="filter-row" id="filterRow" style={location || tag ? '' : 'display: none;'}>
            {/* Hidden inputs to store actual comma-separated values */}
            <input type="hidden" name="location" id="locationInput" value={location || ''} />
            <input type="hidden" name="tag" id="tagInput" value={tag || ''} />

            {/* Custom Tag Inputs */}
            <div class="tag-input-container" id="locationTagContainer">
              <div class="tag-pills" id="locationPills"></div>
              <input
                type="text"
                class="tag-input-field"
                placeholder="Location (e.g. London, Remote)"
                autocomplete="off"
              />
            </div>

            <div class="tag-input-container" id="tagTagContainer">
              <div class="tag-pills" id="tagPills"></div>
              <input
                type="text"
                class="tag-input-field"
                placeholder="Tag (e.g. Internship, React)"
                autocomplete="off"
              />
            </div>
          </div>
        </form>
        <div class="stats-container">
          <div class="stat-pill">
            {total} results
          </div>
          <div class="stat-pill">
            {companyCount} {companyCount === 1 ? 'company' : 'companies'}
          </div>
          <div class="stat-pill">
            {latency}ms
          </div>
        </div>
      </div>

      <div class="content-wrapper">
        <div class="main-content">
          <div class="jobs-grid">
            {jobs.length > 0 ? (
              jobs.map((job) => <JobCard job={job} />)
            ) : (
              <div class="no-results">
                No results found.
              </div>
            )}
          </div>
        </div>

        <aside class="detail-panel" id="detailPanel">
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
            <div
              class="panel-description"
              id="panelDescription"
            >
              <p>Job description will appear here...</p>
            </div>
            <div class="panel-footer">
              <a href="#" target="_blank" class="panel-apply-btn" id="panelApplyBtn">Apply Now</a>
            </div>
          </div>
        </aside>
      </div>
    </>
  )
})

app.get('/api/jobs', async (c) => {
  const query = c.req.query('q')
  const location = c.req.query('location')
  const tag = c.req.query('tag')
  const page = parseInt(c.req.query('page') || '1')

  const { jobs } = await getJobs(c.env.DB, { query, location, tag, page })

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
