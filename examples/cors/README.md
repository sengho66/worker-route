# Cors
Demonstration of returning responses with cors using Worker Route.

## Quick Start
```bash
worker-build --release && npx wrangler dev
```

The dev server should be ready on port 8787.

## Try it out!
#### Profiles
```bash
# Optional parameters are page, sort_by and order_by
# curl http://0.0.0.0:8787/profile; echo
# or
# curl "http://0.0.0.0:8787/profile?page=1"; echo
# or
# curl "http://0.0.0.0:8787/profile?page=1&sort_by=first_name&order_by=desc"; echo
curl http://0.0.0.0:8787/profile; echo
```

#### Single Profile
```bash
# http://0.0.0.0:8787/profile/:/name
curl http://0.0.0.0:8787/profile/toni; echo
```