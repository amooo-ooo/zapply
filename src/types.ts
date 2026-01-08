export interface Job {
    id: number
    title: string
    description?: string
    company: string
    location: string
    city?: string
    region?: string
    country?: string
    country_code?: string
    url: string
    posted: string
    ats: string
    company_url?: string
    tags?: string[]
    departments?: string[]
    degree_levels?: string[]
    subject_areas?: string[]
}
