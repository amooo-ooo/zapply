# zapply

zapply is an internship and early career search engine for students facing the competitive job market. zapply aggregates opportunities and tags relevant roles directly from company ATS systems and other job boards.

<p align="center">
  <img src="public/og-image.png" alt="zapply Screenshot" width="700">
</p>

> [!NOTE]
> Project still in heavy development, expect issues.

## Features

- **High-performance Scraper**: Rust-based scraper for multiple ATS platforms (Greenhouse, Lever, etc.).
- **Tagging**: Regex-based tagging for role types, degree levels, and subject areas.
- **Modern UI**: Fast, mobile-ready frontend built with Hono and Vite.
- **Powered by Cloudflare**: Powered by Cloudflare Workers and D1 for scalable, edge-based performance.

## Tech Stack

- **Frontend**: Hono, Vite, TypeScript, CSS
- **Scraper**: Rust, Tokio, Reqwest, Serde
- **Database**: Cloudflare D1

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) (or npm/node)
- [Rust](https://www.rust-lang.org/) (for the scraper)
- [Wrangler](https://developers.cloudflare.com/workers/wrangler/) (Cloudflare CLI)

### Installation & Development

1. **Install dependencies**:
```bash
bun install
```

2. **Set up the database**:
```bash
wrangler d1 create zapply
bun run db:setup
```

3. **Run the development server**:
```bash
bun run dev
```

### Scraper Setup

The scraper requires geonames data for accurate location normalisation and relational database population. Download the following files from [GeoNames](https://download.geonames.org/export/dump/) and place them in the `scraper/` directory:

- [cities15000.zip](https://download.geonames.org/export/dump/cities15000.zip) (Unzip to get `cities15000.txt`)
- [admin1CodesASCII.txt](https://download.geonames.org/export/dump/admin1CodesASCII.txt)
- [countryInfo.txt](https://download.geonames.org/export/dump/countryInfo.txt)

4. **Run Scraper**:
```bash
bun run scrape:slugs
bun run scrape --limit=1000
```

## Deployment

Deploy to Cloudflare Workers:
```bash
wrangler d1 create zapply --remote
bun run db:setup --remote
bun run scrape:slugs 
bun run scrape --prod
bun run deploy
```
