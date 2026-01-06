import type { Job } from './types'

document.addEventListener('DOMContentLoaded', () => {
    // Theme Toggle Logic
    const themeToggle = document.getElementById('themeToggle')
    const sunIcon = document.querySelector('.sun-icon') as HTMLElement
    const moonIcon = document.querySelector('.moon-icon') as HTMLElement

    const setTheme = (isDark: boolean) => {
        if (isDark) {
            document.documentElement.style.colorScheme = 'dark'
            document.documentElement.classList.add('dark')
        } else {
            document.documentElement.style.colorScheme = 'light'
            document.documentElement.classList.remove('dark')
        }
    }

    // Initialize Theme
    const savedTheme = localStorage.getItem('theme')
    const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches

    // Default to system preference if no saved theme, otherwise use saved
    let isDark = savedTheme ? savedTheme === 'dark' : systemPrefersDark
    setTheme(isDark)

    if (themeToggle) {
        themeToggle.addEventListener('click', () => {
            isDark = !isDark
            setTheme(isDark)
            localStorage.setItem('theme', isDark ? 'dark' : 'light')
        })
    }

    // Filter Toggle
    const filterToggle = document.getElementById('filterToggle')
    if (filterToggle) {
        filterToggle.addEventListener('click', function () {
            const filterRow = document.getElementById('filterRow')
            if (filterRow) {
                if (filterRow.style.display === 'none') {
                    filterRow.style.display = 'grid'
                } else {
                    filterRow.style.display = 'none'
                }
            }
        })
    }

    // --- Smart Tag Input Logic ---
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
            this.input = this.container.querySelector('input') as HTMLInputElement

            if (!this.container || !this.pillsContainer || !this.hiddenInput || !this.input) {
                console.warn(`TagInput: Missing elements for ${containerId}`)
                return
            }

            // Initialize tags from hidden value
            const initialValue = this.hiddenInput.value
            if (initialValue) {
                this.tags = initialValue.split(',').map(t => t.trim()).filter(Boolean)
                this.renderPills()
            }

            this.setupEventListeners()
        }

        setupEventListeners() {
            // Focus input when clicking container
            this.container.addEventListener('click', (e) => {
                if (e.target !== this.input && !(e.target as HTMLElement).closest('.tag-remove')) {
                    this.input.focus()
                }
            })

            // Handle input keys
            this.input.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    if (this.input.value.trim()) {
                        e.preventDefault()
                        this.addTagFromInput()
                    }
                    // If empty, allow default behavior (form submission)
                } else if (e.key === ',') {
                    e.preventDefault()
                    this.addTagFromInput()
                } else if (e.key === 'Backspace' && this.input.value === '' && this.tags.length > 0) {
                    this.removeTag(this.tags.length - 1)
                }
            })

            // Handle blur to add remaining text as tag
            this.input.addEventListener('blur', () => {
                this.addTagFromInput()
            })
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
                    e.stopPropagation() // Prevent container click
                    this.removeTag(index)
                })

                this.pillsContainer.appendChild(pill)
            })
        }
    }

    // Initialize Tag Inputs
    new TagInput('locationTagContainer', 'locationPills', 'locationInput')
    new TagInput('tagTagContainer', 'tagPills', 'tagInput')
    // -----------------------------

    // --- Detail Panel Logic ---
    const detailPanel = document.getElementById('detailPanel')
    const panelDefault = document.getElementById('panelDefault')
    const panelContent = document.getElementById('panelContent')
    const panelJobTitle = document.getElementById('panelJobTitle')
    const panelCompanyName = document.getElementById('panelCompanyName')
    const panelCompanyIcon = document.getElementById('panelCompanyIcon')
    const panelLocation = document.getElementById('panelLocation')?.querySelector('span')
    const panelPosted = document.getElementById('panelPosted')?.querySelector('span')
    const panelTags = document.getElementById('panelTags')
    const panelDescription = document.getElementById('panelDescription')
    const panelApplyBtn = document.getElementById('panelApplyBtn') as HTMLAnchorElement

    let currentActiveCard: HTMLElement | null = null

    const formatDate = (dateString: string) => {
        if (!dateString) return ''
        try {
            const date = new Date(dateString)
            return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).format(date)
        } catch (e) {
            return dateString
        }
    }

    const getTagClass = (name: string) => {
        const hash = name.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
        const classes = ['tag-blue', 'tag-green', 'tag-purple', 'tag-yellow', 'tag-red', 'tag-default']
        return classes[hash % classes.length]
    }

    const showJobDetails = async (jobId: string, cardElement: HTMLElement) => {
        if (!detailPanel || !panelDefault || !panelContent) return

        // Update active state
        if (currentActiveCard) {
            currentActiveCard.classList.remove('active')
        }
        cardElement.classList.add('active')
        currentActiveCard = cardElement

        // Mobile: open panel
        detailPanel.classList.add('open')

        try {
            const res = await fetch(`/api/job/${jobId}`)
            if (!res.ok) throw new Error('Failed to fetch job')

            const job = await res.json() as Job

            // Populate panel
            if (panelJobTitle) panelJobTitle.textContent = job.title || 'Untitled'
            if (panelCompanyName) panelCompanyName.textContent = job.company || 'Unknown'
            if (panelCompanyIcon) panelCompanyIcon.textContent = job.company ? job.company.charAt(0).toUpperCase() : '?'
            if (panelLocation) panelLocation.textContent = job.location || 'Not specified'
            if (panelPosted) panelPosted.textContent = formatDate(job.posted) || 'Unknown'
            if (panelApplyBtn) panelApplyBtn.href = job.url || '#'

            // Tags
            if (panelTags) {
                panelTags.innerHTML = ''
                const allTags = [...(job.departments || []), ...(job.tags || [])]
                allTags.forEach((tag: string) => {
                    const span = document.createElement('span')
                    span.className = `tag ${getTagClass(tag)}`
                    span.textContent = tag
                    panelTags.appendChild(span)
                })
            }

            // Description
            if (panelDescription) {
                if (job.description) {
                    // Some ATS or the database might return escaped HTML entities.
                    // If the first character is '&' or we see common entities, we might need to unescape.
                    // But usually innerHTML handles the first level if it's <p>...
                    // If we see "&lt;p&gt;", we need to unescape it once to get "<p>".
                    let desc = job.description;
                    if (desc.includes('&lt;') || desc.includes('&gt;')) {
                        const temp = document.createElement('div');
                        temp.innerHTML = desc;
                        desc = temp.textContent || temp.innerText || desc;
                    }
                    panelDescription.innerHTML = desc;
                } else {
                    panelDescription.innerHTML = '<p>No description available.</p>';
                }
            }

            // Show content, hide default
            panelDefault.style.display = 'none'
            panelContent.style.display = 'flex'
        } catch (e) {
            console.error('Error loading job details:', e)
        }
    }

    const attachJobCardListeners = (container: Element) => {
        const jobCards = container.querySelectorAll('.job-card[data-job-id]')
        jobCards.forEach((card) => {
            card.addEventListener('click', (e) => {
                // Don't trigger if clicking on the apply button
                if ((e.target as HTMLElement).closest('.apply-btn')) return

                const jobId = card.getAttribute('data-job-id')
                if (jobId) {
                    showJobDetails(jobId, card as HTMLElement)
                }
            })
        })
    }

    // Attach listeners to initial cards
    const mainContent = document.querySelector('.main-content')
    if (mainContent) {
        attachJobCardListeners(mainContent)
    }

    // Export for use in infinite scroll
    (window as any).attachJobCardListeners = attachJobCardListeners
    // -----------------------------

    // Infinite Scroll Logic
    const urlParams = new URLSearchParams(window.location.search)
    let page = parseInt(urlParams.get('page') || '1')
    console.log('Initial page:', page)

    let loading = false
    let hasMore = true
    const grid = document.querySelector('.jobs-grid')

    if (!grid) {
        console.error('Jobs grid not found')
        return
    }

    // Create loading element (Skeleton Container)
    const loadingElement = document.createElement('div')
    loadingElement.className = 'skeleton-container'

    // Create 4 skeleton cards
    const createSkeletonCard = () => {
        return `
      <div class="skeleton-card">
        <div class="skeleton-header">
          <div class="skeleton-icon"></div>
          <div class="skeleton-company"></div>
        </div>
        <div class="skeleton-body">
          <div class="skeleton-title"></div>
          <div class="skeleton-tags">
            <div class="skeleton-tag"></div>
            <div class="skeleton-tag"></div>
            <div class="skeleton-tag"></div>
          </div>
        </div>
        <div class="skeleton-footer">
          <div class="skeleton-ats"></div>
          <div class="skeleton-btn"></div>
        </div>
      </div>
    `
    }

    loadingElement.innerHTML = Array(4).fill(null).map(createSkeletonCard).join('')
    loadingElement.style.display = 'contents' // Use contents so children share the grid

    // Initially hide it by not appending it, or appending hidden. 
    // Since we want to toggle visibility, let's keep it in DOM but hidden. 
    // Or just append/remove. Appending/removing is cleaner for grid.
    // Actually, let's just use a class or style to toggle.
    // The 'contents' display might make toggling 'display: none' tricky if we want to preserve 'display: contents'.
    // Better to append it to the grid when loading, and remove it when done.

    const fetchMoreJobs = async () => {
        if (loading || !hasMore) return
        console.log('Fetching more jobs, page:', page + 1)
        loading = true

        // Append skeleton loader to grid
        grid.appendChild(loadingElement)

        // Get current URL params
        const currentUrlParams = new URLSearchParams(window.location.search)
        currentUrlParams.set('page', (page + 1).toString())

        try {
            const res = await fetch(`/api/jobs?${currentUrlParams.toString()}`)
            if (res.status === 204) {
                hasMore = false
                loading = false
                if (loadingElement.parentNode === grid) {
                    grid.removeChild(loadingElement)
                }
                return
            }

            const html = await res.text()
            if (!html) {
                hasMore = false
            } else {
                const div = document.createElement('div')
                div.innerHTML = html

                // Remove skeletons before appending new items to avoid visual jumpiness or ordering issues
                if (loadingElement.parentNode === grid) {
                    grid.removeChild(loadingElement)
                }

                // Extract valid job cards and append
                Array.from(div.children).forEach((child) => {
                    grid.appendChild(child)
                })

                // Attach click listeners to new cards
                if ((window as any).attachJobCardListeners) {
                    (window as any).attachJobCardListeners(grid)
                }

                page++
            }
        } catch (e) {
            console.error('Error fetching jobs:', e)
        } finally {
            loading = false
            if (loadingElement.parentNode === grid) {
                try {
                    grid.removeChild(loadingElement)
                } catch (e) {
                    // Ignore if already removed
                }
            }

            // Check if we need to fetch more immediately (if sentinel is still visible)
            // This happens if the fetched content isn't enough to fill the screen
            const sentinel = document.getElementById('scroll-sentinel')
            if (hasMore && sentinel) {
                const rect = sentinel.getBoundingClientRect()
                // If sentinel is visible (top is within viewport)
                if (rect.top < window.innerHeight) {
                    console.log('Sentinel still visible, fetching more...')
                    fetchMoreJobs()
                }
            }
        }
    }

    const observer = new IntersectionObserver(
        (entries) => {
            if (entries[0].isIntersecting) {
                fetchMoreJobs()
            }
        },
        {
            root: null,
            rootMargin: '100px', // Fetch before reaching the bottom
            threshold: 0.1,
        }
    )

    // Observer target
    const sentinel = document.createElement('div')
    sentinel.id = 'scroll-sentinel'
    sentinel.style.height = '10px'
    sentinel.style.width = '100%'
    if (grid && grid.parentNode) {
        grid.parentNode.insertBefore(sentinel, grid.nextSibling)
    }

    observer.observe(sentinel)
})
