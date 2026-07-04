/**
 * Lightweight HTML → Markdown converter for agent content negotiation.
 * Strips chrome (nav, footer, scripts, styles) and converts semantic HTML.
 */

const STRIP_SELECTORS = [
  "nav", "footer", "script", "style", "noscript", "svg",
  ".md-header", ".md-sidebar", ".md-footer", ".md-search",
  "[data-md-component=announce]", "[data-md-component=skip]",
];

function decodeEntities(text) {
  return text
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&nbsp;/g, " ");
}

function extractMeta(html, name, property) {
  const patterns = [
    new RegExp(`<meta[^>]+name=["']${name}["'][^>]+content=["']([^"']*)["']`, "i"),
    new RegExp(`<meta[^>]+content=["']([^"']*)["'][^>]+name=["']${name}["']`, "i"),
  ];
  if (property) {
    patterns.push(
      new RegExp(`<meta[^>]+property=["']${property}["'][^>]+content=["']([^"']*)["']`, "i"),
      new RegExp(`<meta[^>]+content=["']([^"']*)["'][^>]+property=["']${property}["']`, "i"),
    );
  }
  for (const re of patterns) {
    const m = html.match(re);
    if (m) return decodeEntities(m[1].trim());
  }
  return "";
}

function stripChrome(html) {
  let out = html;
  for (const sel of STRIP_SELECTORS) {
    if (sel.startsWith(".") || sel.startsWith("[")) {
      const cls = sel.replace(/^\./, "").replace(/\[.*$/, "");
      if (cls) {
        out = out.replace(new RegExp(`<[^>]+class=["'][^"']*\\b${cls}\\b[^"']*["'][^>]*>[\\s\\S]*?<\\/[^>]+>`, "gi"), "");
      }
    } else {
      out = out.replace(new RegExp(`<${sel}\\b[^>]*>[\\s\\S]*?<\\/${sel}>`, "gi"), "");
    }
  }
  out = out.replace(/<!--[\s\S]*?-->/g, "");
  return out;
}

function inlineToMd(text) {
  let s = decodeEntities(text);
  s = s.replace(/<a[^>]+href=["']([^"']*)["'][^>]*>([\s\S]*?)<\/a>/gi, (_, href, label) => {
    const t = label.replace(/<[^>]+>/g, "").trim();
    return t ? `[${t}](${href})` : `[](${href})`;
  });
  s = s.replace(/<strong[^>]*>([\s\S]*?)<\/strong>/gi, "**$1**");
  s = s.replace(/<b[^>]*>([\s\S]*?)<\/b>/gi, "**$1**");
  s = s.replace(/<em[^>]*>([\s\S]*?)<\/em>/gi, "*$1*");
  s = s.replace(/<code[^>]*>([\s\S]*?)<\/code>/gi, "`$1`");
  s = s.replace(/<br\s*\/?>/gi, "\n");
  s = s.replace(/<[^>]+>/g, "");
  return s.replace(/\s+/g, " ").trim();
}

function blockToMd(html) {
  let out = html;
  out = out.replace(/<h1[^>]*>([\s\S]*?)<\/h1>/gi, (_, c) => `\n# ${inlineToMd(c)}\n\n`);
  out = out.replace(/<h2[^>]*>([\s\S]*?)<\/h2>/gi, (_, c) => `\n## ${inlineToMd(c)}\n\n`);
  out = out.replace(/<h3[^>]*>([\s\S]*?)<\/h3>/gi, (_, c) => `\n### ${inlineToMd(c)}\n\n`);
  out = out.replace(/<h4[^>]*>([\s\S]*?)<\/h4>/gi, (_, c) => `\n#### ${inlineToMd(c)}\n\n`);
  out = out.replace(/<pre[^>]*><code[^>]*>([\s\S]*?)<\/code><\/pre>/gi, (_, c) => {
    return `\n\`\`\`\n${decodeEntities(c).replace(/^\n|\n$/g, "")}\n\`\`\`\n\n`;
  });
  out = out.replace(/<pre[^>]*>([\s\S]*?)<\/pre>/gi, (_, c) => {
    return `\n\`\`\`\n${decodeEntities(c.replace(/<[^>]+>/g, "")).replace(/^\n|\n$/g, "")}\n\`\`\`\n\n`;
  });
  out = out.replace(/<li[^>]*>([\s\S]*?)<\/li>/gi, (_, c) => `- ${inlineToMd(c)}\n`);
  out = out.replace(/<p[^>]*>([\s\S]*?)<\/p>/gi, (_, c) => {
    const t = inlineToMd(c);
    return t ? `\n${t}\n\n` : "";
  });
  out = out.replace(/<article[^>]*>([\s\S]*?)<\/article>/gi, (_, c) => blockToMd(c));
  out = out.replace(/<[^>]+>/g, "");
  return out;
}

export function htmlToMarkdown(html, pathname = "/") {
  const title = extractMeta(html, "title") || extractMeta(html, "title", "og:title");
  const description = extractMeta(html, "description") || extractMeta(html, "description", "og:description");

  const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  const articleMatch = html.match(/<article[^>]*>([\s\S]*?)<\/article>/i);
  const body = articleMatch ? articleMatch[1] : (bodyMatch ? bodyMatch[1] : html);
  const content = blockToMd(stripChrome(body));

  const frontmatter = [];
  if (title) frontmatter.push(`title: ${title}`);
  if (description) frontmatter.push(`description: ${description}`);

  let md = "";
  if (frontmatter.length) md += `---\n${frontmatter.join("\n")}\n---\n\n`;
  md += content.trim();
  if (!md) md = `# ${title || pathname}\n\n(No extractable content)`;
  return md.replace(/\n{3,}/g, "\n\n").trim() + "\n";
}

export function estimateTokens(text) {
  return String(Math.ceil(text.length / 4));
}