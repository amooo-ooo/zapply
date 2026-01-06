import type { Job } from './types'

// --- Types ---
interface GlobalState {
    page: number
    loading: boolean
    hasMore: boolean
    currentActiveCard: HTMLElement | null
}

const state: GlobalState = {
    page: 1,
    loading: false,
    hasMore: true,
    currentActiveCard: null
}

// --- DOM Elements ---
const getElements = () => ({
    themeToggle: document.getElementById('themeToggle'),
    filterToggle: document.getElementById('filterToggle'),
    filterRow: document.getElementById('filterRow'),
    grid: document.querySelector('.jobs-grid') as HTMLElement,
    mainContent: document.querySelector('.main-content'),
    detailPanel: document.getElementById('detailPanel'),
    panelDefault: document.getElementById('panelDefault'),
    panelContent: document.getElementById('panelContent'),
    panelJobTitle: document.getElementById('panelJobTitle'),
    panelCompanyName: document.getElementById('panelCompanyName'),
    panelCompanyIcon: document.getElementById('panelCompanyIcon'),
    panelLocation: document.getElementById('panelLocation')?.querySelector('span'),
    panelPosted: document.getElementById('panelPosted')?.querySelector('span'),
    panelTags: document.getElementById('panelTags'),
    panelDescription: document.getElementById('panelDescription'),
    panelApplyBtn: document.getElementById('panelApplyBtn') as HTMLAnchorElement,
    panelDegree: document.getElementById('panelDegree'),
    panelField: document.getElementById('panelField'),
    panelDept: document.getElementById('panelDept'),
    closePanel: document.getElementById('closePanel'),
    panelOverlay: document.getElementById('panelOverlay'),
})

// --- Utilities ---
const formatDate = (dateString: string) => {
    if (!dateString) return ''
    try {
        const date = new Date(dateString)
        return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).format(date)
    } catch (e) {
        return dateString
    }
}

const getTagStyle = (name: string) => {
    const hues = [217, 142, 273, 38, 350, 189, 239, 14]
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    const h = hues[Math.abs(hash) % hues.length];
    return `--tag-hue: ${h};`
}

// --- Tag Input Class ---
class TagInput {
    container: HTMLElement
    pillsContainer: HTMLElement
    input: HTMLInputElement
    hiddenInput: HTMLInputElement
    tags: string[] = []

    constructor(containerId: string, pillsId: string, hiddenInputId: string) {
        this.container = document.getElementById(containerId) as HTMLElement
        this.pillsContainer = document.getElementById(pillsId) as HTMLElement
        this.hiddenInput = document.getElementById(hiddenInputId) as HTMLInputElement
        this.input = this.container?.querySelector('input') as HTMLInputElement

        if (!this.container || !this.pillsContainer || !this.hiddenInput || !this.input) {
            console.warn(`TagInput: Missing elements for ${containerId}`)
            return
        }

        const initialValue = this.hiddenInput.value
        if (initialValue) {
            this.tags = initialValue.split(',').map(t => t.trim()).filter(Boolean)
            this.renderPills()
        }

        this.setupEventListeners()
    }

    setupEventListeners() {
        this.container.addEventListener('click', (e) => {
            if (e.target !== this.input && !(e.target as HTMLElement).closest('.tag-remove')) {
                this.input.focus()
            }
        })

        this.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                if (this.input.value.trim()) {
                    e.preventDefault()
                    this.addTagFromInput()
                }
            } else if (e.key === ',') {
                e.preventDefault()
                this.addTagFromInput()
            } else if (e.key === 'Backspace' && this.input.value === '' && this.tags.length > 0) {
                this.removeTag(this.tags.length - 1)
            }
        })

        this.input.addEventListener('blur', () => this.addTagFromInput())
    }

    addTagFromInput() {
        const text = this.input.value.trim().replace(/,/g, '')
        if (text) {
            this.tags.push(text)
            this.input.value = ''
            this.update()
        }
    }

    removeTag(index: number) {
        this.tags.splice(index, 1)
        this.update()
    }

    update() {
        this.hiddenInput.value = this.tags.join(',')
        this.renderPills()
    }

    renderPills() {
        this.pillsContainer.innerHTML = ''
        this.tags.forEach((tag, index) => {
            const pill = document.createElement('div')
            pill.className = 'tag-pill'
            pill.innerHTML = `
                <span>${tag}</span>
                <div class="tag-remove" data-index="${index}">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </div>
            `
            pill.querySelector('.tag-remove')?.addEventListener('click', (e) => {
                e.stopPropagation()
                this.removeTag(index)
            })
            this.pillsContainer.appendChild(pill)
        })
    }
}

// --- Feature Modules ---
const THEMES: Record<string, any> = {
    neutral: { base: 'zinc', accent: '#3b82f6', preview: '#3f3f46' },
    slate: { base: 'slate', accent: '#6366f1', preview: '#334155' },
    stone: { base: 'stone', accent: '#65a30d', preview: '#44403c' },
    indigo: { base: 'indigo', accent: '#6366f1', preview: '#4338ca' },
    emerald: { base: 'emerald', accent: '#10b981', preview: '#047857' },
    rose: { base: 'rose', accent: '#f43f5e', preview: '#be123c' }
}

const PALETTES: Record<string, any> = {
    zinc: {
        light: '#fafafa', dark: '#09090b',
        cardLight: '#f4f4f5', cardDark: '#18181b',
        mutedLight: '#e4e4e7', mutedDark: '#27272a',
        textLight: '#09090b', textDark: '#fafafa',
        textSecondaryLight: '#3f3f46', textSecondaryDark: '#a1a1aa',
        tagDefaultBgLight: '#f4f4f5', tagDefaultBgDark: '#27272a',
        borderLight: '#e4e4e7', borderDark: '#27272a'
    },
    slate: {
        light: '#f1f5f9', dark: '#020617',
        cardLight: '#e2e8f0', cardDark: '#0f172a',
        mutedLight: '#cbd5e1', mutedDark: '#1e293b',
        textLight: '#0f172a', textDark: '#f8fafc',
        textSecondaryLight: '#334155', textSecondaryDark: '#94a3b8',
        tagDefaultBgLight: '#e2e8f0', tagDefaultBgDark: '#1e293b',
        borderLight: '#cbd5e1', borderDark: '#1e293b'
    },
    stone: {
        light: '#f5f5f4', dark: '#0c0a09',
        cardLight: '#e7e5e4', cardDark: '#1c1917',
        mutedLight: '#d6d3d1', mutedDark: '#292524',
        textLight: '#1c1917', textDark: '#fafaf9',
        textSecondaryLight: '#44403c', textSecondaryDark: '#a8a29e',
        tagDefaultBgLight: '#e7e5e4', tagDefaultBgDark: '#292524',
        borderLight: '#d6d3d1', borderDark: '#292524'
    },
    indigo: {
        light: '#eef2ff', dark: '#030014',
        cardLight: '#e0e7ff', cardDark: '#0a0a23',
        mutedLight: '#c7d2fe', mutedDark: '#1e1b4b',
        textLight: '#1e1b4b', textDark: '#f5f3ff',
        textSecondaryLight: '#4338ca', textSecondaryDark: '#818cf8',
        tagDefaultBgLight: '#e0e7ff', tagDefaultBgDark: '#1e1b4b',
        borderLight: '#c7d2fe', borderDark: '#1e1b4b'
    },
    emerald: {
        light: '#ecfdf5', dark: '#022c22',
        cardLight: '#d1fae5', cardDark: '#064e3b',
        mutedLight: '#a7f3d0', mutedDark: '#065f46',
        textLight: '#064e3b', textDark: '#f0fdf4',
        textSecondaryLight: '#047857', textSecondaryDark: '#34d399',
        tagDefaultBgLight: '#d1fae5', tagDefaultBgDark: '#065f46',
        borderLight: '#6ee7b7', borderDark: '#065f46'
    },
    rose: {
        light: '#fff1f2', dark: '#450a19',
        cardLight: '#ffe4e6', cardDark: '#4c0519',
        mutedLight: '#fecdd3', mutedDark: '#881337',
        textLight: '#881337', textDark: '#fff1f2',
        textSecondaryLight: '#be123c', textSecondaryDark: '#fb7185',
        tagDefaultBgLight: '#ffe4e6', tagDefaultBgDark: '#881337',
        borderLight: '#fda4af', borderDark: '#881337'
    }
}

const applyThemeColors = (themeId: string, isDark: boolean) => {
    const theme = THEMES[themeId] || THEMES.neutral
    const palette = PALETTES[theme.base]
    const root = document.documentElement

    const colors = {
        '--bg-app': isDark ? palette.dark : palette.light,
        '--bg-card': isDark ? palette.cardDark : palette.cardLight,
        '--bg-muted': isDark ? palette.mutedDark : palette.mutedLight,
        '--text-primary': isDark ? palette.textDark : palette.textLight,
        '--text-secondary': isDark ? palette.textSecondaryDark : palette.textSecondaryLight,
        '--tag-default-bg': isDark ? palette.tagDefaultBgDark : palette.tagDefaultBgLight,
        '--border-color': isDark ? palette.borderDark : palette.borderLight,
        '--accent-color': theme.accent
    }

    Object.entries(colors).forEach(([key, val]) => {
        root.style.setProperty(key, val)
    })

    // Update active dot
    document.querySelectorAll('.theme-pill').forEach(dot => {
        dot.classList.toggle('active', dot.getAttribute('data-theme') === themeId)
    })
}

const initTheme = () => {
    const setThemeMode = (isDark: boolean) => {
        document.documentElement.style.colorScheme = isDark ? 'dark' : 'light'
        document.documentElement.classList.toggle('dark', isDark)

        // Parse URL params for theme selection
        const urlParams = new URLSearchParams(window.location.search)
        const urlTheme = urlParams.get('theme')
        const currentAccent = urlTheme || localStorage.getItem('accentTheme') || 'neutral'

        if (urlTheme) {
            localStorage.setItem('accentTheme', urlTheme)
        }

        applyThemeColors(currentAccent, isDark)
    }

    const savedTheme = localStorage.getItem('theme')
    const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    let isDark = savedTheme ? savedTheme === 'dark' : systemPrefersDark

    setThemeMode(isDark)

    const { themeToggle } = getElements()
    themeToggle?.addEventListener('click', () => {
        isDark = !isDark
        setThemeMode(isDark)
        localStorage.setItem('theme', isDark ? 'dark' : 'light')
    })

    // Palette menu toggle
    const paletteToggle = document.getElementById('paletteToggle')
    const themeMenu = document.getElementById('themeMenu')

    paletteToggle?.addEventListener('click', (e) => {
        e.stopPropagation()
        themeMenu?.classList.toggle('open')
    })

    // Close menu when clicking outside
    document.addEventListener('click', (e) => {
        if (themeMenu?.classList.contains('open') && !themeMenu.contains(e.target as Node)) {
            themeMenu.classList.remove('open')
        }
    })

    // Accent theme selection
    document.querySelectorAll('.theme-pill').forEach(dot => {
        dot.addEventListener('click', () => {
            const themeId = dot.getAttribute('data-theme')
            if (themeId) {
                localStorage.setItem('accentTheme', themeId)
                applyThemeColors(themeId, isDark)
                // Optionally close menu after selection
                themeMenu?.classList.remove('open')
            }
        })
    })
}

const initFilters = () => {
    const { filterToggle, filterRow } = getElements()
    const searchForm = document.getElementById('searchForm') as HTMLFormElement

    filterToggle?.addEventListener('click', () => {
        if (filterRow) {
            filterRow.style.display = filterRow.style.display === 'none' ? 'grid' : 'none'
        }
    })

    searchForm?.addEventListener('submit', (e) => {
        e.preventDefault()
        const formData = new FormData(searchForm)
        const params = new URLSearchParams()

        for (const [key, value] of formData.entries()) {
            if (value && value.toString().trim()) {
                params.append(key, value.toString().trim())
            }
        }

        const newUrl = `${window.location.pathname}${params.toString() ? '?' + params.toString() : ''}`
        window.location.href = newUrl
    })
}

const initJobDetails = () => {
    const elements = getElements()

    const showJobDetails = async (jobId: string, cardElement: HTMLElement) => {
        if (!elements.detailPanel || !elements.panelDefault || !elements.panelContent) return

        if (state.currentActiveCard) {
            state.currentActiveCard.classList.remove('active')
        }
        cardElement.classList.add('active')
        state.currentActiveCard = cardElement

        elements.detailPanel.classList.add('open')

        try {
            const res = await fetch(`/api/job/${jobId}`)
            if (!res.ok) throw new Error('Failed to fetch job')

            const job = await res.json() as Job

            if (elements.panelJobTitle) elements.panelJobTitle.textContent = job.title || 'Untitled'
            if (elements.panelCompanyName) elements.panelCompanyName.textContent = job.company || 'Unknown'
            if (elements.panelCompanyIcon) elements.panelCompanyIcon.textContent = job.company ? job.company.charAt(0).toUpperCase() : '?'
            if (elements.panelLocation) elements.panelLocation.textContent = job.location || 'Not specified'
            if (elements.panelPosted) elements.panelPosted.textContent = formatDate(job.posted) || 'Unknown'
            if (elements.panelApplyBtn) elements.panelApplyBtn.href = job.url || '#'

            if (elements.panelDegree) {
                if (job.degree_level) {
                    elements.panelDegree.style.display = 'flex'
                    const span = elements.panelDegree.querySelector('span')
                    if (span) span.textContent = job.degree_level
                } else {
                    elements.panelDegree.style.display = 'none'
                }
            }

            if (elements.panelField) {
                if (job.subject_area) {
                    elements.panelField.style.display = 'flex'
                    const span = elements.panelField.querySelector('span')
                    if (span) span.textContent = job.subject_area
                } else {
                    elements.panelField.style.display = 'none'
                }
            }

            if (elements.panelDept) {
                if (job.departments && job.departments.length > 0) {
                    elements.panelDept.style.display = 'flex'
                    const span = elements.panelDept.querySelector('span')
                    if (span) span.textContent = job.departments.join(', ')
                } else {
                    elements.panelDept.style.display = 'none'
                }
            }

            if (elements.panelTags) {
                elements.panelTags.innerHTML = ''
                const allTags = job.tags || []
                const displayTags = allTags.slice(0, 12)
                displayTags.forEach((tag: string) => {
                    const isRainbow = tag.toUpperCase().includes('LGBTQ')
                    const span = document.createElement('span')
                    span.className = `tag ${isRainbow ? 'tag-rainbow' : ''}`
                    if (!isRainbow) {
                        span.style.cssText = getTagStyle(tag)
                    }
                    span.textContent = tag
                    elements.panelTags!.appendChild(span)
                })

                if (allTags.length > 12) {
                    const moreSpan = document.createElement('span')
                    moreSpan.className = 'tag tag-more'
                    moreSpan.textContent = `+${allTags.length - 12}`
                    elements.panelTags!.appendChild(moreSpan)
                }
            }

            if (elements.panelDescription) {
                elements.panelDescription.innerHTML = job.description || '<p>No description available.</p>'
            }

            elements.panelDefault.style.display = 'none'
            elements.panelContent.style.display = 'flex'

            // Handle mobile specific logic
            if (window.innerWidth <= 850) {
                elements.panelOverlay?.classList.add('active')
                document.body.style.overflow = 'hidden'
            }
        } catch (e) {
            console.error('Error loading job details:', e)
        }
    }

    const closeJobDetails = () => {
        if (!elements.detailPanel) return

        elements.detailPanel.classList.remove('open')
        elements.panelOverlay?.classList.remove('active')
        document.body.style.overflow = ''

        if (state.currentActiveCard) {
            state.currentActiveCard.classList.remove('active')
            state.currentActiveCard = null
        }
    }

    elements.closePanel?.addEventListener('click', closeJobDetails)
    elements.panelOverlay?.addEventListener('click', closeJobDetails)

    const attachListeners = (container: Element) => {
        const jobCards = container.querySelectorAll('.job-card[data-job-id]')
        jobCards.forEach((card) => {
            // Use a flag to prevent multiple attachments
            if ((card as any)._hasListener) return
                ; (card as any)._hasListener = true

            card.addEventListener('click', (e) => {
                if ((e.target as HTMLElement).closest('.apply-btn')) return
                const jobId = card.getAttribute('data-job-id')
                if (jobId) showJobDetails(jobId, card as HTMLElement)
            })
        })
    }

    if (elements.mainContent) {
        attachListeners(elements.mainContent)
    }

    // Export to global for infinite scroll
    ; (window as any).attachJobCardListeners = attachListeners
}

const initInfiniteScroll = () => {
    const { grid } = getElements()
    if (!grid) return

    const urlParams = new URLSearchParams(window.location.search)
    state.page = parseInt(urlParams.get('page') || '1')

    const loadingElement = document.createElement('div')
    loadingElement.className = 'skeleton-container'
    loadingElement.style.display = 'contents'

    const skeletonHtml = `
        <div class="skeleton-card">
            <div class="skeleton-header"><div class="skeleton-icon"></div><div class="skeleton-company"></div></div>
            <div class="skeleton-body">
                <div class="skeleton-title"></div>
                <div class="skeleton-tags"><div class="skeleton-tag"></div><div class="skeleton-tag"></div><div class="skeleton-tag"></div></div>
            </div>
            <div class="skeleton-footer"><div class="skeleton-ats"></div><div class="skeleton-btn"></div></div>
        </div>
    `
    loadingElement.innerHTML = Array(4).fill(skeletonHtml).join('')

    const fetchMoreJobs = async () => {
        if (state.loading || !state.hasMore) return
        state.loading = true
        grid.appendChild(loadingElement)

        const currentUrlParams = new URLSearchParams(window.location.search)
        currentUrlParams.set('page', (state.page + 1).toString())

        try {
            const res = await fetch(`/api/jobs?${currentUrlParams.toString()}`)
            if (res.status === 204) {
                state.hasMore = false
            } else {
                const html = await res.text()
                if (html) {
                    const div = document.createElement('div')
                    div.innerHTML = html
                    Array.from(div.children).forEach(child => grid.appendChild(child))
                    if ((window as any).attachJobCardListeners) {
                        (window as any).attachJobCardListeners(grid)
                    }
                    state.page++
                } else {
                    state.hasMore = false
                }
            }
        } catch (e) {
            console.error('Error fetching jobs:', e)
        } finally {
            state.loading = false
            if (loadingElement.parentNode === grid) grid.removeChild(loadingElement)
            checkSentinelVisibility()
        }
    }

    const checkSentinelVisibility = () => {
        const sentinel = document.getElementById('scroll-sentinel')
        if (state.hasMore && sentinel) {
            const rect = sentinel.getBoundingClientRect()
            if (rect.top < window.innerHeight) fetchMoreJobs()
        }
    }

    const observer = new IntersectionObserver(
        (entries) => {
            if (entries[0].isIntersecting) fetchMoreJobs()
        },
        { root: null, rootMargin: '100px', threshold: 0.1 }
    )

    const sentinel = document.createElement('div')
    sentinel.id = 'scroll-sentinel'
    sentinel.style.height = '10px'
    sentinel.style.width = '100%'
    grid.parentNode?.insertBefore(sentinel, grid.nextSibling)
    observer.observe(sentinel)
}

// --- Main Initialization ---
document.addEventListener('DOMContentLoaded', () => {
    initTheme()
    initFilters()
    new TagInput('locationTagContainer', 'locationPills', 'locationInput')
    new TagInput('tagTagContainer', 'tagPills', 'tagInput')
    new TagInput('sourceTagContainer', 'sourcePills', 'sourceInput')
    new TagInput('fieldTagContainer', 'fieldPills', 'fieldInput')
    initJobDetails()
    initInfiniteScroll()
})

