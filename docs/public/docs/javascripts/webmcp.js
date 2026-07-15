/**
 * WebMCP tools for docs.kruxos.com
 */
(function () {
  if (!navigator.modelContext || typeof navigator.modelContext.registerTool !== "function") return;

  const controller = new AbortController();

  const DOC_PAGES = {
    install: "/quickstart/install/",
    dashboard: "/quickstart/dashboard/",
    mcp: "/developers/api/gateway-mcp/",
    capabilities: "/developers/capabilities/",
    cli: "/quickstart/cli/",
    policies: "/guides/policies/",
    agents: "/guides/managing-agents/",
    rest_api: "/developers/api/rest-api/",
    whitepaper: "/security/whitepaper/",
    getting_started: "/getting-started/",
    llms: "/llms.txt",
    auth: "/auth.md",
  };

  const tools = [
    {
      name: "kruxos_docs_navigate",
      description: "Navigate to a KruxOS documentation page.",
      inputSchema: {
        type: "object",
        properties: {
          page: {
            type: "string",
            enum: Object.keys(DOC_PAGES),
          },
        },
        required: ["page"],
        additionalProperties: false,
      },
      execute: async ({ page }) => {
        const url = DOC_PAGES[page];
        if (url.endsWith(".txt") || url.endsWith(".md")) {
          window.open(url, "_blank");
        } else {
          window.location.href = url;
        }
        return { navigated: url };
      },
    },
    {
      name: "kruxos_docs_list_sections",
      description: "List available KruxOS documentation sections.",
      inputSchema: { type: "object", properties: {}, additionalProperties: false },
      execute: async () => ({ sections: DOC_PAGES }),
    },
    {
      name: "kruxos_get_ports",
      description: "Return KruxOS service ports and auth requirements.",
      inputSchema: { type: "object", properties: {}, additionalProperties: false },
      execute: async () => ({
        ports: {
          7800: { service: "Dashboard", auth: "operator passphrase" },
          7700: { service: "MCP Gateway", auth: "64-char hex API key" },
          7703: { service: "User API", auth: "krx_user_* bearer (loopback)" },
        },
      }),
    },
    {
      name: "kruxos_get_discovery_endpoints",
      description: "Return machine-readable discovery endpoints.",
      inputSchema: { type: "object", properties: {}, additionalProperties: false },
      execute: async () => ({
        endpoints: {
          llmsTxt: "/llms.txt",
          authMd: "/auth.md",
          apiCatalog: "/.well-known/api-catalog",
          openapi: "/.well-known/openapi.json",
          mcpServerCard: "/.well-known/mcp/server-card.json",
          agentSkills: "/.well-known/agent-skills/index.json",
        },
      }),
    },
  ];

  for (const tool of tools) {
    navigator.modelContext.registerTool(tool, { signal: controller.signal });
  }
  window.addEventListener("pagehide", () => controller.abort(), { once: true });
})();