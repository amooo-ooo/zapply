import { Hono } from 'hono'
import {
    Env,
    getJobs,
    getSearchParams,
    JobCard
} from '../index'

const api = new Hono<Env>()

api.get('/jobs', async (c) => {
    const params = getSearchParams(c)
    const { jobs } = await getJobs(c.env.DB, params)

    if (jobs.length === 0) {
        return c.body(null, 204) // No content
    }

    return c.html(
        <>
            {jobs.map((job) => <JobCard job={job} token={c.env.LOGO_DEV_TOKEN || ''} />)}
        </>
    )
})

api.get('/tags/suggestions', async (c) => {
    const query = c.req.query('q')?.toLowerCase() || ''

    try {
        const sql = `
      SELECT name, COUNT(*) as count 
      FROM job_tags 
      WHERE LOWER(name) LIKE ? ESCAPE "\\"
      GROUP BY LOWER(name) 
      ORDER BY count DESC 
      LIMIT 10
    `

        const escapeLike = (str: string) => {
            return str.replace(/[\\%_]/g, '\\$&')
        }

        const results = await c.env.DB.prepare(sql).bind(`%${escapeLike(query)}%`).all()
        const tags = results.results.map((r: any) => r.name)

        return c.json(tags)
    } catch (e) {
        console.error('D1 Error:', e)
        return c.json([], 500)
    }
})

api.get('/sources/suggestions', async (c) => {
    try {
        // Get distinct ATS sources
        const sql = `
      SELECT DISTINCT ats as name
      FROM jobs 
      WHERE ats IS NOT NULL AND ats != ''
      ORDER BY ats
    `

        const results = await c.env.DB.prepare(sql).all()
        // Remove quotes from ATS values (stored as JSON strings)
        const sources = results.results.map((r: any) => r.name.replace(/^"|"$/g, ''))

        return c.json(sources)
    } catch (e) {
        console.error('D1 Error:', e)
        return c.json([], 500)
    }
})

api.get('/companies/suggestions', async (c) => {
    const query = c.req.query('q')?.toLowerCase() || ''

    try {
        const sql = `
      SELECT company, COUNT(*) as count 
      FROM jobs 
      WHERE LOWER(company) LIKE ? ESCAPE "\\"
      GROUP BY LOWER(company) 
      ORDER BY count DESC 
      LIMIT 10
    `

        const escapeLike = (str: string) => {
            return str.replace(/[\\%_]/g, '\\$&')
        }

        const results = await c.env.DB.prepare(sql).bind(`%${escapeLike(query)}%`).all()
        const companies = results.results.map((r: any) => r.company)

        return c.json(companies)
    } catch (e) {
        console.error('D1 Error:', e)
        return c.json([], 500)
    }
})

api.get('/fields/suggestions', async (c) => {
    const query = c.req.query('q')?.toLowerCase() || ''

    try {
        const sql = `
      SELECT name, COUNT(*) as count 
      FROM job_subject_areas 
      WHERE LOWER(name) LIKE ? ESCAPE "\\"
      GROUP BY LOWER(name) 
      ORDER BY count DESC 
      LIMIT 10
    `

        const escapeLike = (str: string) => {
            return str.replace(/[\\%_]/g, '\\$&')
        }

        const results = await c.env.DB.prepare(sql).bind(`%${escapeLike(query)}%`).all()
        const fields = results.results.map((r: any) => r.name)

        return c.json(fields)
    } catch (e) {
        console.error('D1 Error:', e)
        return c.json([], 500)
    }
})

api.get('/job/:id', async (c) => {
    const id = c.req.param('id')

    try {
        // Fetch job metadata and all relations in a single batch
        const [jobRes, tagsRes, deptsRes, degreesRes, subjectsRes] = await c.env.DB.batch([
            c.env.DB.prepare('SELECT * FROM jobs WHERE id = ?').bind(id),
            c.env.DB.prepare('SELECT name FROM job_tags WHERE job_id = ?').bind(id),
            c.env.DB.prepare('SELECT name FROM job_departments WHERE job_id = ?').bind(id),
            c.env.DB.prepare('SELECT name FROM job_degree_levels WHERE job_id = ?').bind(id),
            c.env.DB.prepare('SELECT name FROM job_subject_areas WHERE job_id = ?').bind(id)
        ])

        if (jobRes.results.length === 0) {
            return c.json({ error: 'Job not found' }, 404)
        }

        const job = jobRes.results[0] as any
        job.tags = (tagsRes.results as any[]).map(t => t.name)
        job.departments = (deptsRes.results as any[]).map(d => d.name)
        job.degree_levels = (degreesRes.results as any[]).map(d => d.name)
        job.subject_areas = (subjectsRes.results as any[]).map(s => s.name)

        return c.json(job)
    } catch (e) {
        console.error('D1 Error:', e)
        return c.json({ error: 'Database error' }, 500)
    }
})

export default api
