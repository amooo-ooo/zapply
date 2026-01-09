import { jsxRenderer } from 'hono/jsx-renderer'
import { Link, Script, ViteClient } from 'vite-ssr-components/hono'

declare module 'hono' {
  interface ContextRenderer {
    (content: string | Promise<string>, props?: { logoDevToken?: string }): Response | Promise<Response>
  }
}

export const renderer = jsxRenderer(({ children, logoDevToken }: { children?: any, logoDevToken?: string }) => {
  // @ts-ignore - logoDevToken is passed from c.render
  return (
    <html>
      <head>
        <ViteClient />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
        <Link href="/src/style.css" rel="stylesheet" />
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                try {
                  var savedTheme = localStorage.getItem('theme');
                  var systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                  var isDark = savedTheme ? savedTheme === 'dark' : systemPrefersDark;

                  // Parse URL params for theme selection
                  var urlParams = new URLSearchParams(window.location.search);
                  var urlTheme = urlParams.get('theme');
                  var savedAccent = urlTheme || localStorage.getItem('accentTheme') || 'neutral';
                  
                  if (urlTheme) {
                    localStorage.setItem('accentTheme', urlTheme);
                  }
                  
                  if (isDark) {
                    document.documentElement.style.colorScheme = 'dark';
                    document.documentElement.classList.add('dark');
                  } else {
                    document.documentElement.style.colorScheme = 'light';
                    document.documentElement.classList.remove('dark');
                  }

                  // Pre-define theme colors if possible to avoid flash
                  // This matches the PALETTES constant in client.ts
                  var palettes = {
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
                  };
                  var themes = {
                    neutral: { base: 'zinc', accent: '#3b82f6', preview: '#3f3f46' },
                    slate: { base: 'slate', accent: '#6366f1', preview: '#334155' },
                    stone: { base: 'stone', accent: '#65a30d', preview: '#44403c' },
                    indigo: { base: 'indigo', accent: '#6366f1', preview: '#4338ca' },
                    emerald: { base: 'emerald', accent: '#10b981', preview: '#047857' },
                    rose: { base: 'rose', accent: '#f43f5e', preview: '#be123c' }
                  };

                  var theme = themes[savedAccent] || themes.neutral;
                  var palette = palettes[theme.base];
                  var root = document.documentElement;

                  root.style.setProperty('--bg-app', isDark ? palette.dark : palette.light);
                  root.style.setProperty('--bg-card', isDark ? palette.cardDark : palette.cardLight);
                  root.style.setProperty('--bg-muted', isDark ? palette.mutedDark : palette.mutedLight);
                  root.style.setProperty('--text-primary', isDark ? palette.textDark : palette.textLight);
                  root.style.setProperty('--text-secondary', isDark ? palette.textSecondaryDark : palette.textSecondaryLight);
                  root.style.setProperty('--tag-default-bg', isDark ? palette.tagDefaultBgDark : palette.tagDefaultBgLight);
                  root.style.setProperty('--border-color', isDark ? palette.borderDark : palette.borderLight);
                  root.style.setProperty('--accent-color', theme.accent);

                } catch (e) { console.error('Theme init error', e); }
              })();
            `,
          }}
        />
        <Script src="/src/client.ts" />
        <title>Internship & Early Career Roles | Zapply</title>
        <meta name="description" content="Internship & early career search engine for students." />

        {/* Open Graph / Facebook */}
        <meta property="og:type" content="website" />
        <meta property="og:url" content="https://zapply.amorb.dev/" />
        <meta property="og:title" content="Internship & Early Career Roles | Zapply" />
        <meta property="og:description" content="Internship & early career search engine for students." />
        <meta property="og:image" content="/og-image.png" />

        {/* Twitter */}
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:url" content="https://zapply.amorb.dev/" />
        <meta name="twitter:title" content="Internship & Early Career Roles | Zapply" />
        <meta name="twitter:description" content="Internship & early career search engine for students." />
        <meta name="twitter:image" content="/og-image.png" />
      </head>
      <body data-logo-dev-token={logoDevToken as string}>
        <div class="app-layout">
          {children}
        </div>
      </body>
    </html>
  )
})
