import { existsSync, lstatSync, readlinkSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, relative, resolve, sep } from "node:path";
import { defineConfig, type PageData } from "vitepress";

const siteUrl = "https://konobangu.apeiraco.com";
const repositoryUrl = "https://github.com/apeiraco/konobangu";
const siteRoot = resolve(import.meta.dirname, "..");
const repositoryRoot = resolve(siteRoot, "../..");
const sourceRoot = resolve(repositoryRoot, "docs");
const require = createRequire(import.meta.filename);
const vitepressRequire = createRequire(require.resolve("vitepress"));
const conventionDocuments = [
  "README.md",
  "CHANGELOG.md",
  "CONTRIBUTING.md",
  "SECURITY.md",
  "PRIVACY.md",
];

function assertProjection(link: string, target: string) {
  const expected = relative(dirname(link), target).split(sep).join("/");
  if (
    !lstatSync(link).isSymbolicLink() ||
    readlinkSync(link).split(sep).join("/") !== expected
  ) {
    throw new Error(
      `Enable Git symlinks and restore ${relative(repositoryRoot, link)}; expected ${expected}.`,
    );
  }
}

for (const language of ["en", "zh"]) {
  assertProjection(resolve(siteRoot, language), resolve(sourceRoot, language));
}
assertProjection(
  resolve(siteRoot, "public/assets"),
  resolve(repositoryRoot, "assets"),
);
for (const name of conventionDocuments) {
  const englishSource = resolve(repositoryRoot, name);
  if (!existsSync(englishSource)) continue;
  assertProjection(resolve(sourceRoot, "en", name), englishSource);
  if (!lstatSync(resolve(sourceRoot, "zh", name)).isFile()) {
    throw new Error(`docs/zh/${name} must be a regular source file.`);
  }
}

const documents = [
  { path: "001-DEVELOPMENT-VERIFICATION", en: "Development", zh: "开发与验证" },
  {
    path: "002-AUTHENTICATION-DECISION",
    en: "Authentication",
    zh: "认证与权限",
  },
  {
    path: "003-TASK-DELIVERY-AND-MIGRATION",
    en: "Tasks and migrations",
    zh: "任务与迁移",
  },
  {
    path: "004-MEDIA-AND-CONFIGURATION",
    en: "Media and configuration",
    zh: "媒体与配置",
  },
  {
    path: "roadmap/001-SHORT-TERM-ROADMAP",
    en: "Short-term roadmap",
    zh: "短期路线图",
  },
  {
    path: "roadmap/002-ANIMETA-MODEL",
    en: "Animeta proposal",
    zh: "Animeta 模型提案",
  },
] as const;

function rewriteSourcePath(path: string): string {
  const route = path.replace(/^en\//, "");
  const filename = route.slice(route.lastIndexOf("/") + 1);
  if (!conventionDocuments.includes(filename)) return route;
  return (
    route.slice(0, -filename.length) +
    (filename === "README.md" ? "index.md" : filename.toLowerCase())
  );
}

// Resolve links against the original file, before VitePress removes the English prefix.
function rewriteHref(href: string, source: string): string {
  if (
    !href ||
    href.startsWith("#") ||
    href.startsWith("/") ||
    /^[a-z][a-z0-9+.-]*:/i.test(href)
  )
    return href;
  const separator = href.search(/[?#]/);
  const path = separator < 0 ? href : href.slice(0, separator);
  const suffix = separator < 0 ? "" : href.slice(separator);
  const logicalTarget = resolve(dirname(realpathSync(source)), decodeURI(path));
  const target = existsSync(logicalTarget)
    ? realpathSync(logicalTarget)
    : logicalTarget;
  const repositoryPath = relative(repositoryRoot, target).split(sep).join("/");
  if (conventionDocuments.includes(repositoryPath)) {
    return `/${rewriteSourcePath(`en/${repositoryPath}`)}${suffix}`;
  }
  if (repositoryPath.startsWith("assets/"))
    return `/${repositoryPath}${suffix}`;
  const local = relative(sourceRoot, target).split(sep).join("/");
  if (!local.startsWith("../") && local.endsWith(".md")) {
    return `/${rewriteSourcePath(local)}${suffix}`;
  }
  return repositoryPath.startsWith("../")
    ? href
    : `${repositoryUrl}/${existsSync(target) && lstatSync(target).isDirectory() ? "tree" : "blob"}/master/${repositoryPath}${suffix}`;
}

function editUrl(page: PageData): string {
  return page.frontmatter.sourceEditUrl;
}

function sidebar(language: "en" | "zh") {
  const prefix = language === "en" ? "" : "/zh";
  return [
    {
      text: language === "en" ? "Current contracts" : "当前契约",
      items: documents
        .filter((document) => !document.path.startsWith("roadmap/"))
        .map((document) => ({
          text: document[language],
          link: `${prefix}/${document.path}`,
        })),
    },
    {
      text: language === "en" ? "Plans" : "未来计划",
      items: documents
        .filter((document) => document.path.startsWith("roadmap/"))
        .map((document) => ({
          text: document[language],
          link: `${prefix}/${document.path}`,
        })),
    },
  ];
}

export default defineConfig({
  title: "Konobangu",
  description: "Self-hosted anime subscriptions and recording.",
  srcExclude: ["dist/**", "dist-tsc/**", ".cache/**"],
  outDir: resolve(siteRoot, "dist"),
  cacheDir: resolve(siteRoot, ".cache/vitepress"),
  rewrites: rewriteSourcePath,
  base: "/",
  cleanUrls: true,
  lastUpdated: true,
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" }],
  ],
  sitemap: { hostname: siteUrl },
  transformPageData(page) {
    // Theme callbacks run in the browser; filesystem resolution belongs to the build.
    const source = realpathSync(resolve(siteRoot, page.filePath));
    page.frontmatter.sourceEditUrl = `${repositoryUrl}/edit/master/${relative(repositoryRoot, source).split(sep).join("/")}`;
  },
  transformHead({ pageData }) {
    const route = pageData.relativePath
      .replace(/(?:^|\/)index\.md$/, "/")
      .replace(/\.md$/, "");
    return [
      ["link", { rel: "canonical", href: new URL(route, `${siteUrl}/`).href }],
    ];
  },
  vite: {
    resolve: { preserveSymlinks: true },
    plugins: [
      {
        name: "docs:linked-dependencies",
        configResolved(config) {
          // Preserve document routes, but resolve prebundled dependencies in pnpm's real package directory.
          config.optimizeDeps.include = config.optimizeDeps.include?.map(
            (dependency) =>
              dependency.startsWith("vitepress > ")
                ? vitepressRequire
                    .resolve(dependency.slice("vitepress > ".length))
                    .split(sep)
                    .join("/")
                : dependency,
          );
        },
      },
    ],
  },
  markdown: {
    config(markdown) {
      // Template syntax in configuration examples must remain literal.
      const renderCode = markdown.renderer.rules.code_inline!;
      markdown.renderer.rules.code_inline = (...args) =>
        renderCode(...args).replace("<code", "<code v-pre");
      for (const rule of ["link_open", "image"] as const) {
        const fallback = markdown.renderer.rules[rule]!;
        markdown.renderer.rules[rule] = (
          tokens,
          index,
          options,
          environment,
          renderer,
        ) => {
          const token = tokens[index];
          const attribute = rule === "image" ? "src" : "href";
          const href = token.attrGet(attribute);
          if (href)
            token.attrSet(
              attribute,
              rewriteHref(href, environment.realPath ?? environment.path),
            );
          return fallback(tokens, index, options, environment, renderer);
        };
      }
      // README images retain repository-relative HTML on GitHub and public asset routes on the site.
      for (const rule of ["html_inline", "html_block"] as const) {
        const fallback = markdown.renderer.rules[rule]!;
        markdown.renderer.rules[rule] = (
          tokens,
          index,
          options,
          environment,
          renderer,
        ) =>
          fallback(tokens, index, options, environment, renderer).replace(
            /\b(href|src)=(['"])(.*?)\2/g,
            (_, attribute: string, quote: string, href: string) =>
              `${attribute}=${quote}${rewriteHref(href, environment.realPath ?? environment.path)}${quote}`,
          );
      }
    },
  },
  themeConfig: {
    logo: "/favicon.svg",
    socialLinks: [{ icon: "github", link: repositoryUrl }],
    editLink: { pattern: editUrl },
    search: {
      provider: "local",
      options: {
        locales: {
          zh: {
            translations: {
              button: { buttonText: "搜索文档", buttonAriaLabel: "搜索文档" },
              modal: {
                noResultsText: "没有找到相关结果",
                resetButtonTitle: "清除搜索",
                footer: {
                  selectText: "选择",
                  navigateText: "切换",
                  closeText: "关闭",
                },
              },
            },
          },
        },
      },
    },
  },
  locales: {
    root: {
      label: "English",
      lang: "en",
      themeConfig: {
        nav: [
          { text: "Home", link: "/" },
          { text: "Guide", link: "/001-DEVELOPMENT-VERIFICATION" },
          { text: "Roadmap", link: "/roadmap/001-SHORT-TERM-ROADMAP" },
          { text: "Changelog", link: "/changelog" },
        ],
        sidebar: sidebar("en"),
        editLink: {
          pattern: editUrl,
          text: "Edit this page on GitHub",
        },
      },
    },
    zh: {
      label: "中文",
      lang: "zh-CN",
      link: "/zh/",
      description: "自托管番剧订阅与录制服务。",
      themeConfig: {
        nav: [
          { text: "首页", link: "/zh/" },
          { text: "指南", link: "/zh/001-DEVELOPMENT-VERIFICATION" },
          { text: "路线图", link: "/zh/roadmap/001-SHORT-TERM-ROADMAP" },
          { text: "更新记录", link: "/zh/changelog" },
        ],
        sidebar: sidebar("zh"),
        editLink: {
          pattern: editUrl,
          text: "在 GitHub 上编辑本页",
        },
        outline: { label: "本页内容" },
        docFooter: { prev: "上一页", next: "下一页" },
        lastUpdated: { text: "最后更新" },
        returnToTopLabel: "返回顶部",
        sidebarMenuLabel: "目录",
        darkModeSwitchLabel: "外观",
        langMenuLabel: "语言",
      },
    },
  },
});
