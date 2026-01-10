import fs from 'fs';
import readline from 'readline';
import { Readable } from 'stream';

import { promises as dns } from 'node:dns';

// Configuration & Types

const CONFIG = {
    outputFile: 'scraper/slugs.json',
    indexApiUrl: 'https://index.commoncrawl.org',
    maxIndexesToTry: 4,
    progressInterval: 500,
    minSlugLength: 3,
    concurrency: 50,
    dnsTimeout: 3000,
    bufferSize: 200, // Number of entries before writing to disk
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
        apiUrl: (slug) => `https://boards-api.greenhouse.io/v1/boards/${slug}/jobs?content=true`
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
    domain?: string;
}

// Utilities

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
    readline.createInterface({ input: Readable.fromWeb(body as any), crlfDelay: Infinity });

function extractSlug(url: string, ats: AtsDef): string | null {
    const slug = url.match(ats.pattern)?.[1]?.toLowerCase();
    return slug && slug.length >= CONFIG.minSlugLength && !RESERVED_SLUGS.has(slug) ? slug : null;
}

function formatName(slug: string): string {
    return slug.split(/[-_]/).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

// Core Logic

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

async function getWorkingIndexes(count: number): Promise<string[]> {
    Logger.info('Fetching Common Crawl index metadata...');
    const res = await fetch(`${CONFIG.indexApiUrl}/collinfo.json`);
    if (!res.ok) throw new Error(`Failed to fetch index list: ${res.statusText}`);

    const allIndexes = await res.json() as CCIndex[];
    const workingIndexes: string[] = [];

    for (const index of allIndexes) {
        if (workingIndexes.length >= count) break;
        if (await probeIndex(index.id)) {
            workingIndexes.push(index.id);
        }
    }

    if (workingIndexes.length === 0) {
        throw new Error('All primary Common Crawl indexes are currently unreachable.');
    }

    return workingIndexes;
}

async function extractSlugs(indexId: string, type: AtsType, ats: AtsDef): Promise<Set<string>> {
    const slugs = new Set<string>();
    let totalProcessed = 0;
    let page = 0;

    // Check pagination first
    const pageCheckUrl = `${buildCdxUrl(indexId)}?url=${ats.targetUrl}&matchType=${ats.matchType}&output=json&showNumPages=true`;
    let numPages = 1;

    try {
        const pageRes = await fetch(pageCheckUrl);
        if (pageRes.ok) {
            const info = await pageRes.json() as { pages: number };
            numPages = info.pages || 1;
        }
    } catch {
        // ignore, default to 1
    }

    Logger.info(`Processing ATS: ${type.toUpperCase()} (${numPages} pages)`);

    for (let p = 0; p < numPages; p++) {
        const params = new URLSearchParams({
            url: ats.targetUrl,
            matchType: ats.matchType,
            output: 'json',
            page: p.toString()
        });
        const fullUrl = `${buildCdxUrl(indexId)}?${params}`;

        // ... fetch logic ...
        let attempts = 0;
        const maxAttempts = 3;
        let res: Response | null = null;

        while (attempts < maxAttempts) {
            try {
                res = await fetch(fullUrl);
                if (res.ok) break;
                // Logger.warn(`Attempt ${attempts + 1} failed...`);
            } catch (e) {
                // Logger.warn(...)
            }
            attempts++;
            if (attempts < maxAttempts) await new Promise(r => setTimeout(r, 1000 * Math.pow(2, attempts)));
        }

        if (!res || !res.ok) {
            Logger.warn(`Failed to fetch page ${p} for ${type}. Skipping.`);
            continue;
        }

        if (!res.body) continue;

        const reader = createLineReader(res.body);
        for await (const line of reader) {
            const record = parseCdxLine(line);
            const slug = extractSlug(record?.url || '', ats);
            if (slug) slugs.add(slug);

            if (++totalProcessed % CONFIG.progressInterval === 0) {
                Logger.progress(`Found ${slugs.size.toLocaleString()} companies (scanned ${totalProcessed.toLocaleString()} records)`);
            }
        }
    }

    Logger.success(`${type.toUpperCase()} completed: ${slugs.size.toLocaleString()} unique slugs identified.`);
    return slugs;
}

// Domain Extraction

const NUM_TO_WORD: Record<string, string> = {
    '0': 'zero', '1': 'one', '2': 'two', '3': 'three', '4': 'four',
    '5': 'five', '6': 'six', '7': 'seven', '8': 'eight', '9': 'nine'
};

const WORD_TO_NUM: Record<string, string> = {
    'zero': '0', 'one': '1', 'two': '2', 'three': '3', 'four': '4',
    'five': '5', 'six': '6', 'seven': '7', 'eight': '8', 'nine': '9'
};

async function resolveDomain(slug: string): Promise<string | null> {
    const tlds = ['.com', '.io', '.co', '.ai', '.app', '.dev', '.net', '.org', '.nz', '.co.nz'];
    const variations = [
        (s: string) => s,
        (s: string) => `www.${s}`,
        (s: string) => `get${s}`,
        (s: string) => `try${s}`,
        (s: string) => `use${s}`,
        (s: string) => `${s}app`,
    ];

    const tryResolve = async (candidates: string[]): Promise<string | null> => {
        if (candidates.length === 0) return null;

        const results = await Promise.allSettled(
            candidates.map(async (domain) => {
                const timeoutDetails = new Promise<never>((_, reject) => {
                    setTimeout(() => reject(new Error('Timeout')), CONFIG.dnsTimeout).unref();
                });

                const lookup = async () => {
                    try {
                        await dns.resolve4(domain);
                        return domain;
                    } catch {
                        try {
                            await dns.resolve(domain);
                            return domain;
                        } catch {
                            throw new Error('Not found');
                        }
                    }
                };

                return Promise.race([lookup(), timeoutDetails]);
            })
        );

        const found = results
            .filter((r): r is PromiseFulfilledResult<string> => r.status === 'fulfilled')
            .map(r => r.value);

        return found.length > 0 ? found[0] : null;
    };

    // 1. Generate base candidates in order of priority
    const baseCandidates = new Set<string>();

    // a. Try without dashes first if they exist
    if (slug.includes('-')) {
        baseCandidates.add(slug.replace(/-/g, ''));
    }

    // b. Normal slug
    baseCandidates.add(slug);

    // c. Number variations
    // Handle number-word variations (e.g., "3" <-> "three") and hyphenations
    let numToWordSlug = slug;
    let hasDigits = false;
    for (const [num, word] of Object.entries(NUM_TO_WORD)) {
        if (numToWordSlug.includes(num)) {
            numToWordSlug = numToWordSlug.replace(new RegExp(num, 'g'), word);
            hasDigits = true;
        }
    }
    if (hasDigits) {
        baseCandidates.add(numToWordSlug);
        if (numToWordSlug.includes('-')) baseCandidates.add(numToWordSlug.replace(/-/g, ''));
    }

    let wordToNumSlug = slug;
    let hasWords = false;
    for (const [word, num] of Object.entries(WORD_TO_NUM)) {
        // Match word bound by separators
        const pattern = new RegExp(`(^|[-_])${word}([-_]|$)`, 'g');
        if (pattern.test(wordToNumSlug)) {
            wordToNumSlug = wordToNumSlug.replace(pattern, (match, p1, p2) => {
                // Preserve separators
                return `${p1}${num}${p2}`;
            }).replace(/--/g, '-'); // Clean double dashes
            hasWords = true;
        }
    }
    if (hasWords) {
        baseCandidates.add(wordToNumSlug);
        if (wordToNumSlug.includes('-')) baseCandidates.add(wordToNumSlug.replace(/-/g, ''));
    }

    const uniqueBases = Array.from(baseCandidates);

    // 2. Try Exact Match for all bases with all TLDs
    // Prioritize exact matches over variations
    // NEW: Specifically prioritize .nz and .co.nz for NZ company discovery
    const nzTlds = ['.nz', '.co.nz'];
    const otherTlds = tlds.filter(t => !nzTlds.includes(t));
    const prioritizedTlds = [...nzTlds, ...otherTlds];

    for (const base of uniqueBases) {
        const candidates = prioritizedTlds.map(tld => `${base}${tld}`);
        const found = await tryResolve(candidates);
        if (found) return found;
    }

    // 3. Try Variations (www, get, try, use, app)
    // Try common variations for primary bases
    const priorityBases = uniqueBases.slice(0, 2);
    const secondaryTlds = ['.com', '.io', '.co'];

    for (const base of priorityBases) {
        // Skip base variation (already tried)
        for (let i = 1; i < variations.length; i++) {
            const candidates = secondaryTlds.map(tld => `${variations[i](base)}${tld}`);
            const found = await tryResolve(candidates);
            if (found) return found;
        }
    }

    return null;
}

// Domain Extraction

// Helper to append updates to the cache file (Smart Append)
function saveBatch(entries: CompanyEntry[]) {
    if (entries.length === 0) return;

    if (!fs.existsSync(CONFIG.outputFile)) {
        fs.writeFileSync(CONFIG.outputFile, JSON.stringify(entries, null, 2));
        return;
    }

    const stats = fs.statSync(CONFIG.outputFile);
    const fd = fs.openSync(CONFIG.outputFile, 'r+');

    try {
        const bufferSize = Math.min(1024, stats.size);
        const buffer = Buffer.alloc(bufferSize);
        fs.readSync(fd, buffer, 0, bufferSize, stats.size - bufferSize);

        let lastBracketIndex = -1;
        for (let i = bufferSize - 1; i >= 0; i--) {
            if (buffer[i] === 93) { // ']'
                lastBracketIndex = i;
                break;
            }
        }

        if (lastBracketIndex === -1) {
            Logger.warn('Could not find closing bracket in file to append. Skipping batch write.');
            return;
        }

        const position = stats.size - bufferSize + lastBracketIndex;

        let needsComma = true;
        for (let i = lastBracketIndex - 1; i >= 0; i--) {
            const char = buffer[i];
            if (char === 91) { // '['
                needsComma = false;
                break;
            }
            if (char !== 32 && char !== 10 && char !== 13 && char !== 9) {
                break;
            }
        }

        const newContent = (needsComma ? ',' : '') + JSON.stringify(entries, null, 2).slice(1, -1) + ']';
        fs.writeSync(fd, newContent, position, 'utf-8');
    } finally {
        fs.closeSync(fd);
    }
}

// Driver

async function main() {
    Logger.info('Initializing Zapply Slug Scraper');

    // Simple CLI argument parsing
    const args = process.argv.slice(2);
    const limitArg = args.find(a => a.startsWith('--limit='))?.split('=')[1];
    const typeArg = args.find(a => a.startsWith('--type='))?.split('=')[1];
    const runLimit = limitArg ? parseInt(limitArg) : Infinity;

    try {
        let allEntries: CompanyEntry[] = [];
        const processedSet = new Set<string>();
        const domainCache = new Map<string, string>();
        let pendingEntries: CompanyEntry[] = [];

        if (fs.existsSync(CONFIG.outputFile)) {
            try {
                const raw = fs.readFileSync(CONFIG.outputFile, 'utf-8');
                allEntries = JSON.parse(raw);
                for (const entry of allEntries) {
                    processedSet.add(`${entry.type}:${entry.slug}`);
                    if (entry.domain) {
                        domainCache.set(entry.slug, entry.domain);
                    }
                }
                Logger.info(`Loaded ${allEntries.length} existing entries from cache.`);
            } catch (e) {
                Logger.warn(`Could not read existing cache: ${e}`);
            }
        }

        const indexesToScan = await getWorkingIndexes(3); // Scan last 3 indexes
        Logger.info(`Scanning ${indexesToScan.length} indexes: ${indexesToScan.join(', ')}`);

        let totalNewProcessedAcrossAll = 0;

        for (const indexId of indexesToScan) {
            if (totalNewProcessedAcrossAll >= runLimit) break;
            Logger.info(`\n=== Processing Index: ${indexId} ===`);

            for (const [type, config] of Object.entries(ATS_CONFIGS) as [AtsType, AtsDef][]) {
                if (totalNewProcessedAcrossAll >= runLimit) break;
                if (typeArg && type !== typeArg) continue;


                if (type === 'lever') {
                    Logger.info(`Skipping LEVER (commoncrawl robots.txt restriction)`);
                    continue;
                }

                const slugsFound = await extractSlugs(indexId, type, config);

                // Filter out already processed slugs for THIS type
                const slugsToProcess = Array.from(slugsFound).filter(s => !processedSet.has(`${type}:${s}`));

                if (slugsToProcess.length === 0) {
                    Logger.info(`${type.toUpperCase()} - No NEW slugs found (all ${slugsFound.size} cached for this type).`);
                    continue;
                }

                Logger.info(`Enriching ${slugsToProcess.length} NEW ${type} candidates...`);

                // Chunking for resource efficiency
                const chunkSize = CONFIG.concurrency;
                let totalNewForType = 0;

                for (let i = 0; i < slugsToProcess.length; i += chunkSize) {
                    const remaining = runLimit - totalNewProcessedAcrossAll;
                    if (remaining <= 0) break;

                    const chunk = slugsToProcess.slice(i, i + Math.min(chunkSize, remaining));

                    const newEntries = await Promise.all(chunk.map(async (slug) => {
                        // 1. Try reusing domain if we have it for this slug under another ATS
                        let domain = domainCache.get(slug);

                        // 2. Resolve if not found
                        if (!domain) {
                            domain = await resolveDomain(slug) || undefined;
                        }

                        if (domain) domainCache.set(slug, domain);

                        return {
                            name: formatName(slug),
                            type,
                            slug,
                            board_url: config.boardUrl(slug),
                            api_url: config.apiUrl(slug),
                            domain: domain
                        } as CompanyEntry;
                    }));

                    for (const entry of newEntries) {
                        allEntries.push(entry);
                        processedSet.add(`${entry.type}:${entry.slug}`);
                        pendingEntries.push(entry);
                    }

                    if (pendingEntries.length >= CONFIG.bufferSize) {
                        saveBatch(pendingEntries);
                        pendingEntries = [];
                    }

                    totalNewForType += chunk.length;
                    totalNewProcessedAcrossAll += chunk.length;

                    if (totalNewProcessedAcrossAll % 50 === 0) {
                        process.stdout.write(`\r[PROG] Processed ${totalNewProcessedAcrossAll} new companies (Limit: ${runLimit})...`);
                    }

                    if (totalNewProcessedAcrossAll >= runLimit) {
                        Logger.info(`\nReached run limit of ${runLimit}.`);
                        break;
                    }
                }
                console.log('');
            }

            console.log('');
        }

        // Flush remaining entries
        if (pendingEntries.length > 0) {
            saveBatch(pendingEntries);
            pendingEntries = [];
        }

        console.log('');
        Logger.info(`Finalizing data collection...`);
        Logger.info(`Statistics: ${allEntries.length.toLocaleString()} total companies across ${Object.keys(ATS_CONFIGS).length} ATS sources.`);
        Logger.info(`${totalNewProcessedAcrossAll.toLocaleString()} companies added.`);

        Logger.success(`File ${CONFIG.outputFile} updated successfully.`);

        Logger.info('Scraper operation completed.');
    } catch (err) {
        console.log('');
        Logger.error(`Fatal execution error: ${err instanceof Error ? err.message : String(err)}`);
        process.exit(1);
    }
}

main();
