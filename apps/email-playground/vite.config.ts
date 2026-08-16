import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { render, toPlainText } from "@react-email/render";
import react from "@vitejs/plugin-react";
import { createElement } from "react";
import {
  createServer,
  defineConfig,
  type Plugin,
  type ViteDevServer,
} from "vite";

const root = import.meta.dirname;
const templates = resolve(root, "emails");
const conditions = ["monorepo-tsc", "node", "import", "default"];

function index(names: string[]) {
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Email previews</title></head>
<body><h1>Email previews</h1><ul>${names.map((name) => `<li><a href="templates/${encodeURIComponent(name)}.html">${name}</a> · <a href="templates/${encodeURIComponent(name)}.txt">Plain text</a></li>`).join("")}</ul></body></html>`;
}

async function renderTemplates(server: ViteDevServer) {
  const rendered = new Map<string, string>();
  for (const file of (await readdir(templates)).sort()) {
    if (!file.endsWith(".tsx") || file.startsWith("_")) continue;
    const { default: Template } = await server.ssrLoadModule(`/emails/${file}`);
    const html: string = await render(
      createElement(Template, Template.PreviewProps ?? {}),
    );
    const name = file.slice(0, -4);
    rendered.set(`templates/${name}.html`, html);
    rendered.set(`templates/${name}.txt`, toPlainText(html));
  }
  rendered.set(
    "index.html",
    index(
      [...rendered.keys()]
        .filter((file) => file.endsWith(".html"))
        .map((file) => file.slice(10, -5)),
    ),
  );
  return rendered;
}

// Vite owns template loading; React Email owns email-specific rendering and CSS inlining.
function emailPreview(): Plugin {
  return {
    name: "email-preview",
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        const url = new URL(request.url ?? "/", "http://localhost");
        const path =
          url.pathname === "/"
            ? "index.html"
            : decodeURIComponent(url.pathname.slice(1));
        if (path !== "index.html" && !path.startsWith("templates/"))
          return next();
        try {
          const content = (await renderTemplates(server)).get(path);
          if (content === undefined) return next();
          response.setHeader(
            "Content-Type",
            path.endsWith(".txt")
              ? "text/plain; charset=utf-8"
              : "text/html; charset=utf-8",
          );
          response.end(
            path === "index.html"
              ? await server.transformIndexHtml(url.pathname, content)
              : path.endsWith(".html")
                ? // Email XHTML should not be parsed/reformatted by the HTML5 bundler pipeline.
                  content.replace(
                    "</head>",
                    '<script type="module" src="/@vite/client"></script></head>',
                  )
                : content,
          );
        } catch (error) {
          next(error);
        }
      });
    },
    handleHotUpdate({ server }) {
      server.ws.send({ type: "full-reload" });
      return [];
    },
    generateBundle: {
      order: "post",
      async handler(_options, bundle) {
        const compiler = await createServer({
          configFile: false,
          root,
          plugins: [react()],
          resolve: { conditions },
          ssr: { resolve: { conditions } },
          server: { middlewareMode: true },
        });
        try {
          for (const [fileName, source] of await renderTemplates(compiler)) {
            const existing = bundle[fileName];
            if (existing?.type === "asset") existing.source = source;
            else this.emitFile({ type: "asset", fileName, source });
          }
        } finally {
          await compiler.close();
        }
      },
    },
  };
}

export default defineConfig({
  plugins: [react(), emailPreview()],
  resolve: { conditions },
  ssr: { resolve: { conditions } },
  server: { host: "127.0.0.1", port: 5003 },
  build: { rolldownOptions: { input: resolve(root, "index.html") } },
});
