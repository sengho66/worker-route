# Custom Response
Demonstration of returning custom responses with Worker Route.

## Quick Start
```bash
worker-build --release && npx wrangler dev
```

The dev server should be ready on port 8787.

## Try it out!
#### Bytes Response
```bash
curl http://0.0.0.0:8787/bytes_response; echo
```

#### String Response
```bash
curl http://0.0.0.0:8787/string_response; echo
```

#### Serde Value Response
```bash
curl "http://0.0.0.0:8787/serde_value_response/Foo?age=18"; echo
```

#### Struct Response
```bash
curl http://0.0.0.0:8787/struct_response; echo
```