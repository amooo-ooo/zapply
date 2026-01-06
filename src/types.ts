export interface Job {
    id: number
    title: string
    description?: string
    company: string
    location: string
    url: string
    posted: string
    ats: string
    tags?: string[]
    departments?: string[]
}
