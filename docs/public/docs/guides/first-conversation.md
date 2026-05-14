# First Conversation with KruxOS

Once Claude is connected to KruxOS ([Claude Desktop](../quickstart/connect-claude.md) or [Claude Code](../quickstart/claude-code.md)), here are example prompts that exercise the key capabilities.

## Getting oriented

Start by understanding what's available:

> "What tools do you have available from KruxOS?"

Claude will call `tools/list` and describe the registered capabilities.

> "What agent am I? Show me my identity and session info."

Claude calls `agent.whoami` and `agent.session` to show the authenticated agent name, purpose, policy group, and session metadata.

## Filesystem operations

> "List all files in the workspace."

Calls `filesystem.list` on the workspace root.

> "Create a file called hello.txt with the content 'Hello from KruxOS!'"

Calls `filesystem.write`. This is a great first test — if it works, auth, policy, and the full capability pipeline are functioning.

> "Read hello.txt and show me what's in it."

Calls `filesystem.read` to verify the file was created.

> "Show me the directory structure of the workspace."

Calls `filesystem.list` with `recursive: true`.

## System information

> "What time is it on the server?"

Calls `system.time` — returns UTC time and timezone.

> "Show me the system information — hostname, OS, memory, disk usage."

Calls `system.info` — returns hardware and OS details.

## Process execution

> "Run `echo hello world` and show me the output."

Calls `process.run` — a simple smoke test for process execution.

> "Run `uname -a` to show me the kernel version."

Another simple process call.

## Multi-step workflows

These prompts trigger Claude to chain multiple capability calls:

> "Create a Python file called greet.py that prints 'Hello KruxOS!', then run it and show me the output."

Claude will:
1. `filesystem.write` — create greet.py
2. `process.run` — execute `python3 greet.py`
3. Show the output

> "List the files in the workspace, read any .yaml files you find, and summarize their contents."

Claude will:
1. `filesystem.list` — discover files
2. `filesystem.read` — read each .yaml file
3. Summarize findings

> "Check the system time, system info, and tell me if the server looks healthy."

Claude chains `system.time` and `system.info` calls.

## Policy and governance

> "What policies apply to me? Show me my permission tiers."

Calls `agent.policy` to display the agent's compiled policy rules.

> "What capabilities do I have in the filesystem category?"

Calls `agent.capabilities` with a category filter.

## Prompts that test edge cases

> "Try to read a file that doesn't exist — /workspace/nonexistent.txt"

Should return a structured `NotFound` error with recovery suggestions.

> "What happens if you try to write to /etc/passwd?"

Should be blocked by either policy (filesystem scope) or sandbox restrictions, with a clear error message.

## Tips for good prompts

- **Be specific about paths.** Use full workspace paths (e.g., `/workspace/myfile.txt` or `/data/kruxos/workspace/{agent-name}/myfile.txt` depending on config).
- **Ask Claude to explain what it's doing.** "Read the config and explain each section" works better than "read the config."
- **Chain operations naturally.** "Create a file, verify it exists, then delete it" exercises create → read → delete.
- **Ask about errors.** If a call fails, ask Claude "What went wrong?" — it can interpret the structured error response.

## What to check if something doesn't work

1. **No tools visible?** See [Troubleshooting: Claude doesn't see tools](troubleshooting.md#claude-doesnt-see-any-kruxos-tools)
2. **Auth errors?** Verify your API key with `kruxos agent list`
3. **Permission denied?** Check your agent's policy tier for that capability
4. **Workspace errors?** Ensure the workspace directory exists at the configured path
