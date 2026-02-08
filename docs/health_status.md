# Backend Health Status

Last checked: 2026-02-08 17:56:10 UTC

## Current result

Backend health check is **failing** right now.

- Command run: `curl -sS -f --max-time 5 http://127.0.0.1:8081/`
- Result: `curl: (7) Failed to connect to 127.0.0.1 port 8081`

## How to check

Use this command:

```bash
curl -sS -f --max-time 5 http://127.0.0.1:8081/
```

Healthy backend expected output:

- exit code `0`
- response body: `Hello, world!`

If the backend is down/unreachable, curl returns a non-zero exit code (for example, `7`).
