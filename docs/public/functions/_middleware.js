/**
 * Markdown content negotiation for docs.kruxos.com — no Cloudflare paid feature.
 *
 * 1. Serve mirrored MkDocs source from /markdown/{path}.md (highest quality)
 * 2. HTML → Markdown fallback for any built page
 */

import { htmlToMarkdown, estimateTokens } from "./html-to-markdown.js";

function wantsMarkdown(request) {
  return (request.headers.get("Accept") || "").includes("text/markdown");
}

function normalizePath(pathname) {
  if (pathname === "" || pathname === "/") return "/";
  if (!pathname.endsWith("/") && !pathname.includes(".")) return pathname + "/";
  return pathname;
}

/** Map MkDocs URL /quickstart/install/ → /markdown/quickstart/install.md */
function pathToMarkdown(pathname) {
  if (pathname === "/") return "/markdown/index.md";
  const p = pathname.replace(/\/$/, "");
  return `/markdown${p}.md`;
}

function markdownResponse(text, originalBytes) {
  const headers = {
    "Content-Type": "text/markdown; charset=utf-8",
    "x-markdown-tokens": estimateTokens(text),
    "Vary": "Accept",
    "Cache-Control": "public, max-age=3600",
    "Content-Signal": "ai-train=no, search=yes, ai-input=yes",
  };
  if (originalBytes) {
    headers["x-original-tokens"] = estimateTokens(" ".repeat(Math.ceil(originalBytes / 5)));
  }
  return new Response(text, { status: 200, headers });
}

export async function onRequest(context) {
  if (!wantsMarkdown(context.request)) {
    return context.next();
  }

  const url = new URL(context.request.url);
  const pathname = normalizePath(url.pathname);

  if (/\.(css|js|png|jpg|svg|ico|json|txt|md|woff2?|webmanifest)$/i.test(pathname)) {
    return context.next();
  }

  // 1. Mirrored MkDocs source (canonical)
  const mdPath = pathToMarkdown(pathname);
  const mdAsset = await context.env.ASSETS.fetch(new URL(mdPath, url.origin));
  if (mdAsset.ok) {
    return markdownResponse(await mdAsset.text());
  }

  // 2. HTML fallback
  const htmlResponse = await context.next();
  const ctype = htmlResponse.headers.get("content-type") || "";
  if (!htmlResponse.ok || !ctype.includes("text/html")) {
    return htmlResponse;
  }

  const html = await htmlResponse.text();
  return markdownResponse(htmlToMarkdown(html, pathname), html.length);
}