# Agent Experience: Linux vs KruxOS

The same tasks, side by side. See how KruxOS's typed APIs eliminate the parsing, guessing, and retry loops that consume tokens on traditional Linux.

## Reading a file

=== "Traditional Linux"

    ```python
    # Agent must construct shell command, parse text output, handle encoding
    import subprocess

    result = subprocess.run(
        ["cat", "/workspace/data.csv"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        # What went wrong? Parse stderr to guess
        if "No such file" in result.stderr:
            # Try to find the file
            find_result = subprocess.run(
                ["find", "/workspace", "-name", "data.csv"],
                capture_output=True, text=True
            )
            # Parse find output...
        elif "Permission denied" in result.stderr:
            # Can't tell why — is it a policy? A file permission? SELinux?
            pass
        else:
            # Unknown error — retry? Give up?
            pass

    content = result.stdout  # Hope it's the right encoding
    ```

=== "KruxOS"

    ```python
    result = await agent.capabilities.invoke(
        "filesystem.read",
        path="/workspace/data.csv"
    )

    if result.success:
        content = result.data["content"]
        size = result.data["size_bytes"]
        modified = result.data["modified_at"]
    else:
        # Structured error with recovery suggestions
        match result.error.type:
            case "FileNotFound":
                # Recovery: "Use filesystem.list to see available files"
                pass
            case "PermissionDenied":
                # Recovery: "Use agent.policy to check path access"
                pass
    ```

**Token cost:** ~200 tokens (KruxOS) vs ~500 tokens (Linux). The Linux agent spends tokens on output parsing, error guessing, and retry attempts.

---

## Handling errors

=== "Traditional Linux"

    ```python
    # Agent runs a command that fails
    result = subprocess.run(["python3", "train.py"], capture_output=True, text=True)

    if result.returncode != 0:
        # stderr is a wall of text — agent must parse it
        stderr = result.stderr
        # Is it a missing module? Wrong Python version? Syntax error?
        # OOM? Permission issue? Agent has to pattern-match on raw text.

        if "ModuleNotFoundError" in stderr:
            # Try pip install? Which package? Parse the module name from the traceback
            module = stderr.split("No module named '")[1].split("'")[0]
            subprocess.run(["pip", "install", module])
            # Retry... but was that the right package name?
        elif "PermissionError" in stderr:
            # Try sudo? That's blocked. Try chmod? What permissions?
            pass
        # ... more string parsing
    ```

=== "KruxOS"

    ```python
    try:
        result = await agent.capabilities.invoke(
            "process.run",
            command="python3 train.py"
        )
    except CapabilityError as e:
        # Structured error — no parsing needed
        print(f"Error: {e.type}")       # "ProcessFailed"
        print(f"Exit code: {e.data['exit_code']}")  # 1
        print(f"Stderr: {e.data['stderr']}")  # Full stderr preserved

        for recovery in e.recovery:
            print(f"Try: {recovery.action}")
            # "retry" — "Retry the command"
            # "check_dependencies" — "Verify required modules are installed"
    ```

**Token cost:** ~150 tokens (KruxOS) vs ~800 tokens (Linux). Error recovery on Linux burns tokens on pattern matching, wrong guesses, and retry loops.

---

## Discovering available tools

=== "Traditional Linux"

    ```python
    # Agent doesn't know what it can do. Trial and error:
    result = subprocess.run(["which", "git"], capture_output=True, text=True)
    has_git = result.returncode == 0

    result = subprocess.run(["which", "docker"], capture_output=True, text=True)
    has_docker = result.returncode == 0

    # Check version to guess available flags
    result = subprocess.run(["git", "--version"], capture_output=True, text=True)
    # Parse "git version 2.43.0" → is this new enough for sparse-checkout?

    # Read man pages for usage — thousands of tokens
    result = subprocess.run(["man", "git-log"], capture_output=True, text=True)
    # 5000+ tokens of man page text...
    ```

=== "KruxOS"

    ```python
    # Discover everything available, with schemas
    caps = await agent.capabilities.list()
    # Returns: 86 capabilities with purpose, inputs, outputs

    # Get details for one capability
    cap = await agent.capabilities.describe("git.log")
    print(cap.purpose)       # "Returns the commit log..."
    print(cap.when_to_use)   # "Use git.log to see recent commits..."
    for inp in cap.inputs:
        print(f"  {inp.name}: {inp.type}")  # Typed parameters
    ```

**Token cost:** ~100 tokens (KruxOS) vs ~5000+ tokens (Linux). Man pages and `--help` output are massive. KruxOS provides exactly what the agent needs.

---

## Working with secrets

=== "Traditional Linux"

    ```python
    import os

    # Secret is in an environment variable — agent can see it
    api_key = os.environ.get("OPENWEATHER_API_KEY")
    # Agent now has the raw secret. It could:
    # - Log it accidentally
    # - Include it in an error report
    # - Send it to another service
    # - Store it in persistent state

    response = requests.get(
        f"https://api.openweathermap.org/data/2.5/weather?q=London&appid={api_key}"
    )
    ```

=== "KruxOS"

    ```python
    # Agent never sees the secret — use-not-read model
    result = await agent.capabilities.invoke(
        "weather.current",
        location="London"
    )
    # The capability implementation accessed the secret internally
    # via the vault. The agent only gets the weather data back.

    # Even if the agent tries to read the secret directly:
    result = await agent.capabilities.invoke(
        "secrets.use", name="OPENWEATHER_API_KEY"
    )
    # Returns: {"injected": true, "capability": "weather.current"}
    # NOT the actual secret value
    ```

**Security:** On Linux, the agent has the raw secret. On KruxOS, secrets are injected into capability execution environments — the agent never sees them.

---

## Multi-step workflow

=== "Traditional Linux"

    ```python
    # Search for files, read them, create a report — lots of text parsing

    # Step 1: Find Python files
    result = subprocess.run(
        ["find", "/workspace", "-name", "*.py", "-type", "f"],
        capture_output=True, text=True
    )
    files = result.stdout.strip().split("\n")
    # Hope there are no filenames with newlines...

    # Step 2: Count lines in each file
    total_lines = 0
    for f in files:
        result = subprocess.run(
            ["wc", "-l", f], capture_output=True, text=True
        )
        # Parse "   42 /workspace/main.py" — split on whitespace
        count = int(result.stdout.strip().split()[0])
        total_lines += count

    # Step 3: Write report
    report = f"Found {len(files)} Python files, {total_lines} lines total\n"
    with open("/workspace/report.txt", "w") as fh:
        fh.write(report)
    ```

=== "KruxOS"

    ```python
    # Step 1: Search for Python files
    result = await agent.capabilities.invoke(
        "filesystem.search",
        directory="/workspace",
        pattern="*.py"
    )
    files = result.data["matches"]  # Structured list

    # Step 2: Get file stats
    total_lines = 0
    for f in files:
        stat = await agent.capabilities.invoke(
            "filesystem.stat", path=f["path"]
        )
        total_lines += stat.data["line_count"]

    # Step 3: Write report
    report = f"Found {len(files)} Python files, {total_lines} lines total\n"
    await agent.capabilities.invoke(
        "filesystem.write",
        path="/workspace/report.txt",
        content=report
    )
    # Write is soft-deleted recoverable for 24 hours
    ```

**Token cost:** ~1,200 tokens (KruxOS) vs ~3,000 tokens (Linux). Every shell command requires text parsing and error checking that consumes tokens.

---

## Sending email safely

=== "Traditional Linux (Docker)"

    ```python
    import smtplib
    from email.mime.text import MIMEText

    # Direct SMTP — no safety net
    msg = MIMEText("Quarterly report attached.")
    msg["Subject"] = "Q1 Report"
    msg["From"] = "agent@company.com"
    msg["To"] = "all-staff@company.com"  # Oops — sent to ALL staff

    # Sent immediately. No undo. No approval. No buffer.
    with smtplib.SMTP("smtp.gmail.com", 587) as server:
        server.starttls()
        server.login(os.environ["GMAIL_USER"], os.environ["GMAIL_PASS"])
        server.send_message(msg)  # Gone. Can't take it back.
    ```

=== "KruxOS"

    ```python
    result = await agent.capabilities.invoke(
        "email.send",
        to="all-staff@company.com",
        subject="Q1 Report",
        body="Quarterly report attached."
    )

    # Not sent yet — buffered for 5 minutes
    print(f"Will send at: {result.data['buffer_until']}")
    print(f"Cancel with: proxy.cancel_write(write_id='{result.data['write_id']}')")

    # Supervisor gets notified and can cancel
    # Batch protection kicks in at 20 sends/hour
    # Rollback point created automatically

    # If it was a mistake:
    await agent.capabilities.invoke(
        "proxy.cancel_write",
        write_id=result.data["write_id"]
    )
    # Email never sent. Crisis averted.
    ```

**Safety:** On Linux, sends are immediate and irrevocable. On KruxOS, every write to an external service is buffered, cancellable, batch-protected, and rollback-enabled.

---

## Summary

| Dimension | Traditional Linux | KruxOS |
|-----------|------------------|---------|
| **Interface** | Shell commands + text parsing | Typed APIs + structured responses |
| **Error handling** | Parse stderr text, guess the cause | Typed errors with recovery actions |
| **Discovery** | `which`, `--help`, `man` (thousands of tokens) | Schema-aware capability listing |
| **Secrets** | Environment variables (visible to agent) | Use-not-read vault (never exposed) |
| **External services** | Direct API access (immediate, irrevocable) | Buffered, cancellable, rollback-enabled |
| **Token efficiency** | Baseline | ~60% fewer tokens |
| **Task completion** | ~65-85% | ~90-98% |
| **Audit** | Container logs (deletable) | Hash-chained, tamper-proof, queryable |

The difference is not about what agents *can* do — it's about how efficiently and safely they do it.
