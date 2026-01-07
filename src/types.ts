export interface Job {
    id: number
    title: string
    description?: string
    company: string
    location: string
    url: string
    posted: string
    ats: string
    company_url?: string
    tags?: string[]
    departments?: string[]
    degree_levels?: string[]
    subject_areas?: string[]
    // Backwards compatibility
    degree_level?: string
    subject_area?: string
}
