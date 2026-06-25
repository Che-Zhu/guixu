# Docker

Guixu can run as a single Docker container. The image builds the Rust API as a release binary and runs it as a non-root user.

## Build

```bash
docker build -t guixu:local .
```

## Run

```bash
docker run --rm \
  --name guixu \
  -p 3000:3000 \
  -e YOUZHIYOUXING_COOKIE="_weasley_key=replace-with-deployed-cookie" \
  guixu:local
```

The container defaults to:

```env
PORT=3000
RUST_LOG=guixu=info
```

`YOUZHIYOUXING_COOKIE` is required at runtime. Do not bake `.env` files or real upstream cookies into the image. Pass secrets through the runtime environment supplied by the deployment platform.

On Railway, do not set `GUIXU_BIND_ADDR` to `127.0.0.1:3000`. Railway injects `PORT`, and Guixu binds to `0.0.0.0:$PORT` automatically so the public proxy can reach the container.

## Smoke Check

In another terminal:

```bash
curl -i http://127.0.0.1:3000/healthz
```

Expected response:

```http
HTTP/1.1 200 OK
```

Expected body:

```text
ok
```

## Youzhiyouxing API

With a valid `YOUZHIYOUXING_COOKIE`, the Youzhiyouxing summary endpoint is available at:

```bash
curl -i http://127.0.0.1:3000/youzhiyouxing
```

If the upstream session is expired or the cookie is invalid, Guixu returns:

```http
HTTP/1.1 502 Bad Gateway
content-type: application/json
```

```json
{
  "error": "upstream_session_expired",
  "message": "Youzhiyouxing session is expired or invalid. Refresh YOUZHIYOUXING_COOKIE."
}
```
