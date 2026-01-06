import fs from 'fs';
import readline from 'readline';
import { Readable } from 'stream';
import type { ReadableStream } from 'stream/web';

// --- Configuration & Types ---

const CONFIG = {
    outputFile: 'slugs.json',
    indexApiUrl: 'https://index.commoncrawl.org',
    maxIndexesToTry: 4,
    progressInterval: 250,
    minSlugLength: 3,
} as const;

const RESERVED_SLUGS = new Set([
    'embed', 'frames', 'internal', 'api', 'admin', 'interface', 's', 'v1', 'jobs', 'robots',
    'www', 'blog', 'help', 'support', 'app', 'status', 'assets', 'static', 'cdn',
    'privacy', 'terms', 'cookie', 'legal'
]);

type AtsType = 'greenhouse' | 'lever' | 'smartrecruiters' | 'ashby' | 'workable' | 'recruitee' | 'breezy';

interface AtsDef {
    targetUrl: string;
    matchType: 'prefix' | 'domain';
    pattern: RegExp;
    boardUrl: (slug: string) => string;
    apiUrl: (slug: string) => string;
}

const ATS_CONFIGS: Record<AtsType, AtsDef> = {
    greenhouse: {
        targetUrl: 'boards.greenhouse.io',
        matchType: 'prefix',
        pattern: /boards\.greenhouse\.io\/([a-zA-Z0-9_-]+)/,
        boardUrl: (slug) => `https://boards.greenhouse.io/${slug}`,
        apiUrl: (slug) => `https://boards-api.greenhouse.io/v1/boards/${slug}/jobs`
    },
    lever: {
        targetUrl: 'jobs.lever.co',
        matchType: 'prefix',
        pattern: /jobs\.lever\.co\/([a-zA-Z0-9_-]+)/,
        boardUrl: (slug) => `https://jobs.lever.co/${slug}`,
        apiUrl: (slug) => `https://api.lever.co/v0/postings/${slug}`
    },
    smartrecruiters: {
        targetUrl: 'jobs.smartrecruiters.com',
        matchType: 'prefix',
        pattern: /jobs\.smartrecruiters\.com\/([a-zA-Z0-9_-]+)/,
        boardUrl: (slug) => `https://jobs.smartrecruiters.com/${slug}`,
        apiUrl: (slug) => `https://api.smartrecruiters.com/v1/companies/${slug}/postings`
    },
    ashby: {
        targetUrl: 'jobs.ashbyhq.com',
        matchType: 'prefix',
        pattern: /jobs\.ashbyhq\.com\/([a-zA-Z0-9_-]+)/,
        boardUrl: (slug) => `https://jobs.ashbyhq.com/${slug}`,
        apiUrl: (slug) => `https://api.ashbyhq.com/posting-api/job-board/${slug}`
    },
    workable: {
        targetUrl: 'apply.workable.com',
        matchType: 'prefix',
        pattern: /apply\.workable\.com\/([a-zA-Z0-9_-]+)/,
        boardUrl: (slug) => `https://apply.workable.com/${slug}`,
        apiUrl: (slug) => `https://apply.workable.com/api/v1/widget/accounts/${slug}`
    },
    recruitee: {
        targetUrl: 'recruitee.com',
        matchType: 'domain',
        pattern: /([a-zA-Z0-9_-]+)\.recruitee\.com/,
        boardUrl: (slug) => `https://${slug}.recruitee.com`,
        apiUrl: (slug) => `https://${slug}.recruitee.com/api/offers`
    },
    breezy: {
        targetUrl: 'breezy.hr',
        matchType: 'domain',
        pattern: /([a-zA-Z0-9_-]+)\.breezy\.hr/,
        boardUrl: (slug) => `https://${slug}.breezy.hr`,
        apiUrl: (slug) => `https://${slug}.breezy.hr/json`
    }
};

interface CCIndex { id: string; name: string }
interface CdxRecord { url: string; status?: string; timestamp?: string }

interface CompanyEntry {
    name: string;
    type: AtsType;
    slug: string;
    board_url: string;
    api_url: string;
}

// --- Utilities ---

class Logger {
    static info(msg: string) {
        console.log(`[INFO] ${msg}`);
    }

    static status(msg: string) {
        process.stdout.write(`[WAIT] ${msg}`);
    }

    static success(msg: string) {
        this.clear();
        console.log(`[DONE] ${msg}`);
    }

    static warn(msg: string) {
        console.warn(`[WARN] ${msg}`);
    }

    static error(msg: string) {
        console.error(`[FAIL] ${msg}`);
    }

    static progress(msg: string) {
        this.clear();
        process.stdout.write(`[PROG] ${msg}`);
    }

    static clear() {
        if (process.stdout.isTTY) {
            readline.clearLine(process.stdout, 0);
            readline.cursorTo(process.stdout, 0);
        } else {
            process.stdout.write('\r\x1b[K');
        }
    }
}

const buildCdxUrl = (indexId: string) => `${CONFIG.indexApiUrl}/${indexId}-index`;

const parseCdxLine = (line: string): CdxRecord | null => {
    try { return JSON.parse(line) } catch { return null }
};

const createLineReader = (body: ReadableStream<Uint8Array>) =>
    readline.createInterface({ input: Readable.fromWeb(body), crlfDelay: Infinity });

function extractSlug(url: string, ats: AtsDef): string | null {
    const slug = url.match(ats.pattern)?.[1]?.toLowerCase();
    return slug && slug.length >= CONFIG.minSlugLength && !RESERVED_SLUGS.has(slug) ? slug : null;
}

function formatName(slug: string): string {
    return slug.split(/[-_]/).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

// --- Core Logic ---

async function probeIndex(indexId: string): Promise<boolean> {
    const testUrl = `${buildCdxUrl(indexId)}?url=google.com&limit=1&output=json`;
    Logger.status(`Testing index ${indexId}... `);
    try {
        const res = await fetch(testUrl);
        if (res.ok) {
            Logger.success(`Index ${indexId} is ONLINE`);
            return true;
        }
        Logger.error(`Index ${indexId} returned STATUS ${res.status}`);
        return false;
    } catch {
        Logger.error(`Index ${indexId} connection FAILED`);
        return false;
    }
}

async function findWorkingIndex(): Promise<string> {
    Logger.info('Fetching Common Crawl index metadata...');
    const res = await fetch(`${CONFIG.indexApiUrl}/collinfo.json`);
    if (!res.ok) throw new Error(`Failed to fetch index list: ${res.statusText}`);

    const indexes = await res.json() as CCIndex[];
    for (const index of indexes.slice(0, CONFIG.maxIndexesToTry)) {
        if (await probeIndex(index.id)) return index.id;
    }
    throw new Error('All primary Common Crawl indexes are currently unreachable.');
}

async function extractSlugs(indexId: string, type: AtsType, ats: AtsDef): Promise<Set<string>> {
    const params = new URLSearchParams({ url: ats.targetUrl, matchType: ats.matchType, output: 'json' });
    const fullUrl = `${buildCdxUrl(indexId)}?${params}`;

    Logger.info(`Processing ATS: ${type.toUpperCase()}`);

    const res = await fetch(fullUrl);
    if (!res.ok) {
        Logger.warn(`Source fetch failed for ${type}: ${res.status} ${res.statusText}`);
        return new Set();
    }

    const slugs = new Set<string>();
    let totalProcessed = 0;

    if (!res.body) return slugs;

    for await (const line of createLineReader(res.body)) {
        const record = parseCdxLine(line);
        const slug = extractSlug(record?.url || '', ats);
        if (slug) slugs.add(slug);

        if (++totalProcessed % CONFIG.progressInterval === 0) {
            Logger.progress(`Found ${slugs.size.toLocaleString()} companies (scanned ${totalProcessed.toLocaleString()} records)`);
        }
    }

    Logger.success(`${type.toUpperCase()} completed: ${slugs.size.toLocaleString()} unique slugs identified.`);
    return slugs;
}

// --- Driver ---

async function main() {
    Logger.info('Initializing Zapply Slug Scraper');
    console.log('');

    try {
        const indexId = await findWorkingIndex();
        const allEntries: CompanyEntry[] = [];

        for (const [type, config] of Object.entries(ATS_CONFIGS) as [AtsType, AtsDef][]) {
            if (type === 'lever') {
                Logger.info(`Skipping LEVER (commoncrawl robots.txt restriction)`);
                continue;
            }

            const slugs = await extractSlugs(indexId, type, config);
            for (const slug of slugs) {
                allEntries.push({
                    name: formatName(slug),
                    type,
                    slug,
                    board_url: config.boardUrl(slug),
                    api_url: config.apiUrl(slug)
                });
            }
        }

        console.log('');
        Logger.info(`Finalizing data collection...`);
        Logger.info(`Statistics: ${allEntries.length.toLocaleString()} total companies across ${Object.keys(ATS_CONFIGS).length} ATS sources.`);

        allEntries.sort((a, b) => a.name.localeCompare(b.name));

        Logger.status(`Writing output to ${CONFIG.outputFile}... `);
        fs.writeFileSync(CONFIG.outputFile, JSON.stringify(allEntries, null, 2));
        Logger.success(`File ${CONFIG.outputFile} written successfully.`);

        Logger.info('Scraper operation completed.');
    } catch (err) {
        console.log('');
        Logger.error(`Fatal execution error: ${err instanceof Error ? err.message : String(err)}`);
        process.exit(1);
    }
}

main();


