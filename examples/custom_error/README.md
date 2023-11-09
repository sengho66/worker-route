# Custom Error
Demonstration of returning custom errors with Worker Route.

## Quick Start
```bash
worker-build --release && npx wrangler dev
```

The dev server should be ready on port 8787.

## Try it out!
```bash
curl http://0.0.0.0:8787/error; echo
```

```bash
curl http://0.0.0.0:8787/error-json; echo
```