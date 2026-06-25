# Dockerfile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a production-oriented Docker image for the Guixu Rust API service.

**Architecture:** Use a multi-stage Docker build: the builder stage compiles the locked Rust release binary, and the runtime stage contains only Debian slim, CA certificates, a non-root user, and the `guixu` binary. The container defaults `GUIXU_BIND_ADDR` to `0.0.0.0:3000` so the Axum service is reachable through Docker port publishing, while `YOUZHIYOUXING_COOKIE` remains an injected runtime secret.

**Tech Stack:** Docker, Rust 2021, Cargo, Axum, Debian bookworm-slim.

---

## Scope

Build:

- A root `Dockerfile` for building and running the Guixu API.
- A root `.dockerignore` that keeps local secrets, git data, and build artifacts out of the Docker context.
- Docker usage documentation under `docs/docker.md`.
- Local verification commands for Cargo tests, image build, and container health check.

Do not build:

- Docker Compose.
- Kubernetes manifests.
- Registry publishing automation.
- Runtime embedding of `.env` or real cookies.
- A Docker healthcheck that adds extra runtime dependencies.

## File Structure

- `Dockerfile`: Multi-stage image definition. Builder compiles `target/release/guixu`; runtime installs CA certificates, runs as a non-root user, exposes port `3000`, and starts `guixu`.
- `.dockerignore`: Docker build context exclusions for `.git`, `target`, local env files, editor noise, and superpowers execution plans.
- `docs/docker.md`: Human-facing Docker build/run/smoke-test instructions.

## Runtime Contract

The container listens on:

```text
0.0.0.0:3000
```

Required runtime env:

```env
YOUZHIYOUXING_COOKIE="_weasley_key=replace-with-deployed-cookie"
```

Optional runtime env:

```env
GUIXU_BIND_ADDR=0.0.0.0:3000
RUST_LOG=guixu=info
```

Published local URL when running with `-p 3003:3000`:

```text
http://127.0.0.1:3003/healthz
http://127.0.0.1:3003/youzhiyouxing
```

---

### Task 1: Add Docker Build Context Rules

**Files:**
- Create: `.dockerignore`

- [ ] **Step 1: Verify Docker context rules are missing**

Run:

```bash
test ! -f .dockerignore
```

Expected: command succeeds because `.dockerignore` does not exist yet.

- [ ] **Step 2: Create `.dockerignore`**

Create `.dockerignore`:

```dockerignore
.git
.gitignore
target
.env
.env.*
!.env.example
.DS_Store
docs/superpowers/plans
```

- [ ] **Step 3: Verify `.dockerignore` keeps secrets excluded**

Run:

```bash
sed -n '1,120p' .dockerignore
```

Expected output includes:

```text
.env
.env.*
!.env.example
```

- [ ] **Step 4: Commit Docker context rules**

Run:

```bash
git add .dockerignore
git commit -m "chore: add docker build context rules"
```

Expected: commit succeeds.

### Task 2: Add Production Dockerfile

**Files:**
- Create: `Dockerfile`

- [ ] **Step 1: Verify Docker image build fails before Dockerfile exists**

Run:

```bash
docker build -t guixu:local .
```

Expected: FAIL with a message equivalent to:

```text
failed to read dockerfile
```

- [ ] **Step 2: Create `Dockerfile`**

Create `Dockerfile`:

```dockerfile
# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 10001 --create-home --shell /usr/sbin/nologin guixu

COPY --from=builder /app/target/release/guixu /usr/local/bin/guixu

USER guixu

ENV GUIXU_BIND_ADDR=0.0.0.0:3000
ENV RUST_LOG=guixu=info

EXPOSE 3000

CMD ["guixu"]
```

- [ ] **Step 3: Build the Docker image**

Run:

```bash
docker build -t guixu:local .
```

Expected: PASS. Final output includes:

```text
Successfully tagged guixu:local
```

If Docker uses BuildKit, the equivalent expected result is:

```text
naming to docker.io/library/guixu:local
```

- [ ] **Step 4: Commit Dockerfile**

Run:

```bash
git add Dockerfile
git commit -m "build: add guixu dockerfile"
```

Expected: commit succeeds.

### Task 3: Document Docker Build and Run

**Files:**
- Create: `docs/docker.md`

- [ ] **Step 1: Create Docker documentation**

Create `docs/docker.md`:

````markdown
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
GUIXU_BIND_ADDR=0.0.0.0:3000
RUST_LOG=guixu=info
```

Do not bake `.env` files or real upstream cookies into the image. Pass secrets through the runtime environment supplied by the deployment platform.

## Smoke Check

In another terminal:

```bash
curl -i http://127.0.0.1:3000/healthz
```

Expected response:

```http
HTTP/1.1 200 OK
```

```text
ok
```

The Youzhiyouxing endpoint is:

```http
GET http://127.0.0.1:3000/youzhiyouxing
```

If `YOUZHIYOUXING_COOKIE` is expired or invalid, the endpoint returns `502 Bad Gateway` with:

```json
{
  "error": "upstream_session_expired",
  "message": "Youzhiyouxing session is expired or invalid. Refresh YOUZHIYOUXING_COOKIE."
}
```
````

- [ ] **Step 2: Verify Docker documentation renders as plain Markdown**

Run:

```bash
sed -n '1,220p' docs/docker.md
```

Expected: output contains `docker build -t guixu:local .`, `docker run --rm`, `/healthz`, and `/youzhiyouxing`.

- [ ] **Step 3: Commit Docker documentation**

Run:

```bash
git add docs/docker.md
git commit -m "docs: document docker usage"
```

Expected: commit succeeds.

### Task 4: Verify Docker Runtime

**Files:**
- No file changes expected.

- [ ] **Step 1: Run Rust format check**

Run:

```bash
cargo fmt --check
```

Expected: PASS.

- [ ] **Step 2: Run Rust tests**

Run:

```bash
cargo test
```

Expected: PASS. The API route test must use static fixtures and must not call real Youzhiyouxing.

- [ ] **Step 3: Build Docker image from a clean context**

Run:

```bash
docker build --no-cache -t guixu:local .
```

Expected: PASS. Final output tags `guixu:local`.

- [ ] **Step 4: Start container with an invalid but syntactically valid cookie**

Run:

```bash
docker run -d --rm \
  --name guixu-docker-smoke \
  -p 3003:3000 \
  -e YOUZHIYOUXING_COOKIE="_weasley_key=invalid" \
  guixu:local
```

Expected: command prints a container id.

- [ ] **Step 5: Check container health endpoint**

Run:

```bash
curl -i --max-time 10 http://127.0.0.1:3003/healthz
```

Expected response includes:

```http
HTTP/1.1 200 OK
```

Expected body:

```text
ok
```

- [ ] **Step 6: Check startup logs**

Run:

```bash
docker logs guixu-docker-smoke
```

Expected output includes:

```text
starting guixu
```

- [ ] **Step 7: Stop smoke container**

Run:

```bash
docker stop guixu-docker-smoke
```

Expected: command prints:

```text
guixu-docker-smoke
```

- [ ] **Step 8: Inspect git status**

Run:

```bash
git status --short
```

Expected: working tree is clean.

## Self-Review

- Spec coverage: The plan adds a root Dockerfile, excludes local secrets/build artifacts from Docker context, documents build/run commands, and verifies the container can serve `GET /healthz`.
- Placeholder scan: No unresolved placeholders such as `TBD`, `TODO`, or vague "add appropriate handling" steps remain. The only `replace-with-deployed-cookie` value is an explicit safe example, not a missing implementation detail.
- Type consistency: Runtime env names match the Rust config module: `GUIXU_BIND_ADDR` and `YOUZHIYOUXING_COOKIE`. The exposed container port matches the Dockerfile `EXPOSE 3000` and docs examples.
