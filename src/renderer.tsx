import { jsxRenderer } from 'hono/jsx-renderer'
import { Link, ViteClient } from 'vite-ssr-components/hono'

export const renderer = jsxRenderer(({ children }) => {
  return (
    <html>
      <head>
        <ViteClient />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet" />
        <Link href="/src/style.css" rel="stylesheet" />
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                try {
                  var savedTheme = localStorage.getItem('theme');
                  var systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                  var isDark = savedTheme ? savedTheme === 'dark' : systemPrefersDark;
                  if (isDark) {
                    document.documentElement.style.colorScheme = 'dark';
                    document.documentElement.classList.add('dark');
                  } else {
                    document.documentElement.style.colorScheme = 'light';
                    document.documentElement.classList.remove('dark');
                  }
                } catch (e) { console.error('Theme init error', e); }
              })();
            `,
          }}
        />
        <script type="module" src="/src/client.ts"></script>
        <title>Zapply | Early Career Roles</title>
      </head>
      <body>
        <div class="app-layout">
          {children}
        </div>
      </body>
    </html>
  )
})
