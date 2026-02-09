# Complete Docker Build & GitHub Workflow Documentation

This document explains in exhaustive detail how the Docker container build and GitHub Container Registry (GHCR) publishing workflow operates for the dockge-plus project.

---

## Table of Contents

1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [The Build Process Flow](#the-build-process-flow)
4. [Dockerfile Architecture](#dockerfile-architecture)
5. [GitHub Workflow Deep Dive](#github-workflow-deep-dive)
6. [Environment Variables & Secrets](#environment-variables--secrets)
7. [Multi-Architecture Building](#multi-architecture-building)
8. [Frontend Build Process](#frontend-build-process)
9. [Cache Strategy](#cache-strategy)
10. [Common Issues & Solutions](#common-issues--solutions)

---

## 1. Overview

This project uses a **multi-stage Docker build** strategy combined with **GitHub Actions** to automatically build and publish Docker images to GitHub Container Registry. The workflow supports:

- **Multi-architecture builds**: linux/amd64, linux/arm64, linux/arm/v7
- **Automatic tagging**: Based on git tags or manual input
- **Layer caching**: Using GitHub Actions cache
- **Pre-built base images**: Reduces build time
- **Compiled healthcheck binary**: Written in Go for efficiency

---

## 2. Project Structure

```
dockge-plus/
├── backend/                    # Node.js/TypeScript backend code
│   ├── index.ts               # Entry point (starts DockgeServer)
│   ├── dockge-server.ts       # Main server implementation
│   └── ...                    # Other server modules
├── frontend/                   # Vue.js frontend source
│   ├── src/                   # Vue components and app code
│   ├── vite.config.ts         # Vite build configuration
│   └── index.html             # HTML entry point
├── frontend-dist/             # Built frontend (generated during build)
├── docker/                     # All Dockerfiles
│   ├── Base.Dockerfile        # Base image with system dependencies
│   ├── Dockerfile             # Main multi-stage build
│   └── BuildHealthCheck.Dockerfile  # Go healthcheck builder
├── extra/
│   └── healthcheck.go         # Go source for healthcheck binary
├── .dockerignore              # Files to exclude from Docker context
├── .github/workflows/
│   └── publish-ghcr.yml       # GitHub Actions workflow
├── package.json               # Node.js dependencies & scripts
└── package-lock.json          # Locked dependency versions
```

---

## 3. The Build Process Flow

### High-Level Flow

```
1. Trigger Event (git tag push or manual workflow_dispatch)
   ↓
2. GitHub Actions starts workflow
   ↓
3. Checkout repository code
   ↓
4. Setup QEMU (for multi-arch emulation)
   ↓
5. Setup Docker Buildx (advanced builder)
   ↓
6. Login to GitHub Container Registry (GHCR)
   ↓
7. Setup Node.js 22
   ↓
8. Install npm dependencies (npm clean-install)
   ↓
9. Build frontend with Vite → creates frontend-dist/
   ↓
10. Determine image tags (based on event type)
   ↓
11. Build multi-arch Docker image (3 platforms in parallel)
    - Uses pre-built base image
    - Copies frontend-dist into image
    - Installs production dependencies
    - Copies backend code
   ↓
12. Push image to ghcr.io
   ↓
13. Make package public (API call)
   ↓
14. Report success with pull command
```

---

## 4. Dockerfile Architecture

### 4.1 Base Image (`docker/Base.Dockerfile`)

**Purpose**: Creates a base image with all system-level dependencies installed. This is built **separately** and pushed to Docker Hub to speed up main builds.

```dockerfile
FROM node:22-bookworm-slim
RUN apt update && apt install --yes --no-install-recommends \
    curl \
    ca-certificates \
    gnupg \
    unzip \
    dumb-init \                          # Proper init system for containers
    && install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg \
    && chmod a+r /etc/apt/keyrings/docker.gpg \
    && echo \
         "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
         "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
         tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt update \
    && apt --yes --no-install-recommends install \
         docker-ce-cli \                 # Docker CLI (for managing containers)
         docker-compose-plugin \         # Docker Compose v2
    && rm -rf /var/lib/apt/lists/* \    # Clean up apt cache
    && npm install -g tsx               # TypeScript executor
```

**Key Points**:
- Based on Debian Bookworm (slim variant for smaller size)
- Installs Docker CLI and Compose plugin (dockge manages Docker containers)
- Installs `dumb-init` (prevents zombie processes in containers)
- Installs `tsx` globally (runs TypeScript directly)
- Cleans up apt lists to reduce image size

**Build command** (from package.json):
```bash
docker buildx build \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  -t louislam/dockge:base \
  -f ./docker/Base.Dockerfile . \
  --push
```

---

### 4.2 Healthcheck Builder (`docker/BuildHealthCheck.Dockerfile`)

**Purpose**: Compiles the Go healthcheck binary for all architectures.

```dockerfile
FROM golang:1.21.4-bookworm
WORKDIR /app
ARG TARGETPLATFORM                      # Provided by buildx (e.g., linux/amd64)
COPY ./extra/healthcheck.go ./extra/healthcheck.go
RUN go build -x -o ./extra/healthcheck ./extra/healthcheck.go
```

**Key Points**:
- Uses Go 1.21.4 compiler
- `TARGETPLATFORM` arg allows cross-compilation
- Produces a single binary at `/app/extra/healthcheck`
- Binary is statically linked (no runtime dependencies)

**Build command** (from package.json):
```bash
docker buildx build \
  -f docker/BuildHealthCheck.Dockerfile \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  -t louislam/dockge:build-healthcheck . \
  --push
```

---

### 4.3 Main Dockerfile (`docker/Dockerfile`)

This is a **multi-stage build** with 3 stages:

#### Stage 1: Healthcheck Binary (`build_healthcheck`)

```dockerfile
FROM louislam/dockge:build-healthcheck AS build_healthcheck
```

- Pulls the pre-built healthcheck binary from Docker Hub
- This stage is just a reference for copying the binary later

#### Stage 2: Production Dependencies (`build`)

```dockerfile
FROM louislam/dockge:base AS build
WORKDIR /app
COPY --chown=node:node ./package.json ./package.json
COPY --chown=node:node ./package-lock.json ./package-lock.json
RUN npm ci --omit=dev
```

**Key Points**:
- `npm ci` = clean install (uses package-lock.json exactly)
- `--omit=dev` = only installs production dependencies
- `--chown=node:node` = files owned by non-root user
- This creates a clean `/app/node_modules` with only production deps

#### Stage 3: Release Image (`release`)

```dockerfile
FROM louislam/dockge:base AS release
WORKDIR /app

# Copy healthcheck binary from stage 1
COPY --chown=node:node --from=build_healthcheck /app/extra/healthcheck /app/extra/healthcheck

# Copy production node_modules from stage 2
COPY --from=build /app/node_modules /app/node_modules

# Copy all application code
COPY --chown=node:node . .

# Create data directory
RUN mkdir ./data

# Workaround for node-pty issue
ENV UV_USE_IO_URING=0

VOLUME /app/data
EXPOSE 5001
HEALTHCHECK --interval=60s --timeout=30s --start-period=60s --retries=5 CMD extra/healthcheck
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["tsx", "./backend/index.ts"]
```

**Key Points**:
- Copies compiled healthcheck binary from `build_healthcheck` stage
- Copies production node_modules from `build` stage
- Copies **entire application directory** (filtered by .dockerignore)
- `COPY . .` includes:
  - `backend/` (TypeScript backend code)
  - `frontend-dist/` (pre-built frontend from GitHub Actions)
  - `common/` (shared TypeScript code)
  - `extra/` (utility scripts)
  - `package.json` and `package-lock.json`
- `VOLUME /app/data` = persisted storage for stacks/config
- `HEALTHCHECK` = Docker runs `extra/healthcheck` every 60s
- `dumb-init` = proper signal handling and zombie reaping
- `tsx ./backend/index.ts` = runs TypeScript directly without compilation

#### Stage 4: Nightly Variant (`nightly`)

```dockerfile
FROM release AS nightly
RUN npm run mark-as-nightly
```

- Based on `release` stage
- Marks the version as nightly build
- Used for `louislam/dockge:nightly` tag

---

### 4.4 .dockerignore File

**Purpose**: Excludes files from Docker build context (speeds up build and reduces image size)

```
# Should be identical to .gitignore
.env
node_modules
.idea
data
stacks
tmp
/private

# Docker extra (not in .gitignore)
docker           # Don't copy Dockerfiles into image
frontend         # Don't copy frontend source (we use frontend-dist)
.editorconfig
.eslintrc.cjs
.git
.gitignore
README.md
```

**Critical Points**:
- `node_modules` is excluded (dependencies installed inside Docker)
- `frontend` is excluded (we build it first, then copy `frontend-dist`)
- `docker/` is excluded (don't need Dockerfiles in the image)
- `.git` is excluded (reduces context size significantly)

---

## 5. GitHub Workflow Deep Dive

### File: `.github/workflows/publish-ghcr.yml`

#### 5.1 Trigger Events

```yaml
on:
  push:
    tags:
      - 'v*'                    # Triggered on version tags (e.g., v1.5.0)
  workflow_dispatch:            # Manual trigger from GitHub UI
    inputs:
      tag:
        description: 'Tag to publish (e.g., latest, v1.5.0)'
        required: false
        default: 'latest'
```

**Two ways to trigger**:
1. **Automatic**: Push a git tag starting with 'v' (e.g., `git tag v1.5.0 && git push --tags`)
2. **Manual**: Go to GitHub Actions → "Publish to GitHub Container Registry" → "Run workflow"

---

#### 5.2 Permissions

```yaml
permissions:
  contents: read               # Read repository content
  packages: write              # Write to GitHub Packages (GHCR)
```

**Why needed**:
- `contents: read` = checkout code
- `packages: write` = push Docker images to ghcr.io

---

#### 5.3 Job Configuration

```yaml
jobs:
  publish:
    runs-on: ubuntu-latest
    timeout-minutes: 120       # 2 hour timeout
```

---

#### 5.4 Step-by-Step Breakdown

##### Step 1: Checkout Code

```yaml
- name: Checkout code
  uses: actions/checkout@v4
```

- Clones the git repository into the runner
- Uses latest stable checkout action

##### Step 2: Setup QEMU

```yaml
- name: Set up QEMU
  uses: docker/setup-qemu-action@v3
```

**What is QEMU?**
- **Q**uick **EMU**lator - virtualizes different CPU architectures
- Allows building ARM images on x86_64 runners
- Essential for multi-architecture builds

**Without QEMU**: Can only build for the runner's native architecture (linux/amd64)
**With QEMU**: Can build for linux/amd64, linux/arm64, linux/arm/v7 simultaneously

##### Step 3: Setup Docker Buildx

```yaml
- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3
```

**What is Buildx?**
- Docker's next-generation build system
- Supports:
  - Multi-platform builds
  - Build caching
  - Parallel stage execution
  - BuildKit backend (faster builds)

**Default Docker vs Buildx**:
| Feature | docker build | docker buildx build |
|---------|--------------|---------------------|
| Multi-platform | ❌ | ✅ |
| Advanced caching | ❌ | ✅ |
| Parallel builds | ❌ | ✅ |
| Build secrets | Limited | ✅ |

##### Step 4: Login to GHCR

```yaml
- name: Login to GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.repository_owner }}
    password: ${{ secrets.GITHUB_TOKEN }}
```

**Authentication Details**:
- `registry: ghcr.io` = GitHub Container Registry
- `username` = Repository owner (e.g., "SamVellaUK")
- `password` = Automatically provided `GITHUB_TOKEN` secret
  - Scoped to the repository
  - Has `packages: write` permission (from workflow permissions)
  - Expires after workflow completes

**Why GITHUB_TOKEN works**:
- GitHub automatically creates this secret for every workflow run
- No need to create personal access tokens (PATs)
- Permissions controlled by workflow `permissions` block

##### Step 5: Setup Node.js

```yaml
- name: Set up Node.js 22
  uses: actions/setup-node@v4
  with:
    node-version: 22
```

- Installs Node.js 22.x (matches engines in package.json)
- Required for building frontend

##### Step 6: Install Dependencies

```yaml
- name: Install dependencies
  run: npm clean-install --no-fund
```

**`npm clean-install` (alias: `npm ci`)**:
- Deletes existing node_modules
- Installs exact versions from package-lock.json
- Faster than `npm install` in CI
- Fails if package.json and package-lock.json are out of sync

**`--no-fund`**:
- Suppresses "please fund these packages" messages
- Cleaner CI logs

##### Step 7: Build Frontend

```yaml
- name: Build frontend
  run: npm run build:frontend
```

**What happens**:
```bash
# From package.json:
vite build --config ./frontend/vite.config.ts
```

**Vite build process** (frontend/vite.config.ts):
```typescript
{
  root: "./frontend",              // Source directory
  build: {
    outDir: "../frontend-dist"     // Output directory (project root)
  }
}
```

**Output**:
- Creates `frontend-dist/` directory with:
  - `index.html` (optimized)
  - `assets/` (JS, CSS, fonts, images)
  - JS bundles (minified, tree-shaken)
  - CSS (minified)
  - Compressed versions (gzip + brotli)

**Vite optimizations applied**:
- Code splitting (dynamic imports)
- Tree shaking (removes unused code)
- Minification (Terser for JS, cssnano for CSS)
- Asset hashing (for cache busting)
- Compression (gzip + brotli via vite-plugin-compression)

##### Step 8: Set Lowercase Owner

```yaml
- name: Set lowercase owner name
  id: lowercase
  run: |
    echo "owner=$(echo '${{ github.repository_owner }}' | tr '[:upper:]' '[:lower:]')" >> $GITHUB_OUTPUT
```

**Why?**
- Docker image names must be lowercase
- `github.repository_owner` might be "SamVellaUK"
- `tr '[:upper:]' '[:lower:]'` converts to "samvellauk"
- Sets output variable `owner` for later steps

**Usage in later steps**:
```yaml
ghcr.io/${{ steps.lowercase.outputs.owner }}/dockge-plus
```

##### Step 9: Extract Version/Tags

```yaml
- name: Extract version from tag
  id: version
  run: |
    if [ "${{ github.event_name }}" == "push" ]; then
      VERSION=${GITHUB_REF#refs/tags/v}
      echo "version=${VERSION}" >> $GITHUB_OUTPUT
      echo "tags=ghcr.io/${{ steps.lowercase.outputs.owner }}/dockge-plus:${VERSION},ghcr.io/${{ steps.lowercase.outputs.owner }}/dockge-plus:latest" >> $GITHUB_OUTPUT
    else
      TAG="${{ github.event.inputs.tag }}"
      echo "version=${TAG}" >> $GITHUB_OUTPUT
      echo "tags=ghcr.io/${{ steps.lowercase.outputs.owner }}/dockge-plus:${TAG}" >> $GITHUB_OUTPUT
    fi
```

**Logic**:

**Scenario A: Git tag push** (`github.event_name == "push"`):
```bash
# Input: git tag v1.5.0 && git push --tags
# GITHUB_REF = "refs/tags/v1.5.0"
VERSION=${GITHUB_REF#refs/tags/v}  # Strips "refs/tags/v" → "1.5.0"
tags=ghcr.io/samvellauk/dockge-plus:1.5.0,ghcr.io/samvellauk/dockge-plus:latest
```

**Scenario B: Manual workflow_dispatch**:
```bash
# User inputs "latest" in GitHub UI
TAG="latest"
tags=ghcr.io/samvellauk/dockge-plus:latest
```

**Output variables**:
- `version`: The version string (e.g., "1.5.0" or "latest")
- `tags`: Comma-separated list of full image tags

##### Step 10: Docker Metadata

```yaml
- name: Docker metadata
  id: meta
  uses: docker/metadata-action@v5
  with:
    images: ghcr.io/${{ steps.lowercase.outputs.owner }}/dockge-plus
    labels: |
      org.opencontainers.image.title=Dockge Plus
      org.opencontainers.image.description=A fancy, easy-to-use and reactive self-hosted docker compose.yaml stack-oriented manager
      org.opencontainers.image.vendor=${{ github.repository_owner }}
```

**What are OCI labels?**
- Open Container Initiative standard metadata
- Embedded in Docker image manifest
- Visible with `docker inspect`

**Example output** (`docker inspect`):
```json
{
  "Labels": {
    "org.opencontainers.image.title": "Dockge Plus",
    "org.opencontainers.image.description": "A fancy, easy-to-use...",
    "org.opencontainers.image.vendor": "SamVellaUK",
    "org.opencontainers.image.created": "2026-02-09T09:56:53Z",
    "org.opencontainers.image.source": "https://github.com/SamVellaUK/dockge-plus",
    "org.opencontainers.image.revision": "0e23e1b..."
  }
}
```

##### Step 11: Build and Push Docker Image

```yaml
- name: Build and push Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    file: ./docker/Dockerfile
    platforms: linux/amd64,linux/arm64,linux/arm/v7
    push: true
    tags: ${{ steps.version.outputs.tags }}
    labels: ${{ steps.meta.outputs.labels }}
    target: release
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

**Parameter breakdown**:

- **`context: .`**
  - Build context = entire repository root
  - Filtered by `.dockerignore`

- **`file: ./docker/Dockerfile`**
  - Path to Dockerfile

- **`platforms: linux/amd64,linux/arm64,linux/arm/v7`**
  - Builds 3 architectures in parallel
  - Creates a multi-arch manifest (single tag, multiple images)

- **`push: true`**
  - Automatically push to registry after build
  - `false` = build only (for testing)

- **`tags: ${{ steps.version.outputs.tags }}`**
  - Example: `ghcr.io/samvellauk/dockge-plus:1.5.0,ghcr.io/samvellauk/dockge-plus:latest`
  - Applies all tags to the manifest

- **`labels: ${{ steps.meta.outputs.labels }}`**
  - Applies OCI labels from metadata step

- **`target: release`**
  - Builds the `release` stage from Dockerfile
  - Skips the `nightly` stage

- **`cache-from: type=gha`**
  - Restore cache from GitHub Actions cache
  - Speeds up builds by reusing layers

- **`cache-to: type=gha,mode=max`**
  - Save cache to GitHub Actions cache
  - `mode=max` = cache all layers (not just final stage)

**Build process (internally)**:

1. **Load cache** from GitHub Actions cache storage
2. **Build for each platform** (parallelized by BuildKit):
   ```
   Platform: linux/amd64
   ├── Stage: build_healthcheck → Pull louislam/dockge:build-healthcheck
   ├── Stage: build → npm ci --omit=dev
   └── Stage: release → Assemble final image

   Platform: linux/arm64
   ├── (same stages, cross-compiled via QEMU)
   └── ...

   Platform: linux/arm/v7
   ├── (same stages, cross-compiled via QEMU)
   └── ...
   ```

3. **Create manifest** (multi-arch index):
   ```
   ghcr.io/samvellauk/dockge-plus:latest
   ├── linux/amd64 → sha256:abc123...
   ├── linux/arm64 → sha256:def456...
   └── linux/arm/v7 → sha256:ghi789...
   ```

4. **Push manifest + images** to ghcr.io

5. **Save cache** for next build

##### Step 12: Make Package Public

```yaml
- name: Make package public
  run: |
    gh api \
      --method PATCH \
      -H "Accept: application/vnd.github+json" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      /users/${{ steps.lowercase.outputs.owner }}/packages/container/dockge-plus \
      -f visibility='public' || echo "Package may already be public or doesn't exist yet"
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**What this does**:
- Uses GitHub CLI (`gh`) to call REST API
- Endpoint: `PATCH /users/{owner}/packages/container/{package_name}`
- Sets `visibility` to `public`

**Why needed?**
- New packages default to **private** visibility
- Private packages require authentication to pull
- Public packages can be pulled anonymously

**Error handling**:
- `|| echo "..."` = Don't fail workflow if API call fails
- Failures happen if:
  - Package is already public
  - Package doesn't exist yet (first build)
  - API rate limit exceeded

##### Step 13: Success Report

```yaml
- name: Image published
  run: |
    echo "✓ Docker image published to GitHub Container Registry"
    echo "✓ Package visibility set to public"
    echo "Tags: ${{ steps.version.outputs.tags }}"
    echo ""
    echo "Pull with: docker pull ghcr.io/${{ steps.lowercase.outputs.owner }}/dockge-plus:latest"
```

- Prints success message to workflow logs
- Shows tags that were pushed
- Provides pull command for users

---

## 6. Environment Variables & Secrets

### Workflow Variables

| Variable | Source | Example Value | Usage |
|----------|--------|---------------|-------|
| `github.repository_owner` | GitHub context | `SamVellaUK` | Image namespace |
| `github.event_name` | GitHub context | `push` or `workflow_dispatch` | Determines tag logic |
| `github.event.inputs.tag` | Workflow input | `latest` | Manual tag input |
| `GITHUB_REF` | GitHub context | `refs/tags/v1.5.0` | Extracts version |
| `GITHUB_TOKEN` | Automatic secret | `ghs_...` | Auth for GHCR & API |

### Dockerfile Variables

| Variable | Set In | Default | Purpose |
|----------|--------|---------|---------|
| `TARGETPLATFORM` | Buildx (automatic) | `linux/amd64` | Cross-compilation target |
| `UV_USE_IO_URING` | Dockerfile | `0` | Disables io_uring (node-pty fix) |
| `DOCKGE_HOST` | Runtime | `127.0.0.1` | Healthcheck hostname |
| `DOCKGE_PORT` | Runtime | `5001` | Healthcheck port |
| `DOCKGE_SSL_KEY` | Runtime | (none) | SSL key path |
| `DOCKGE_SSL_CERT` | Runtime | (none) | SSL cert path |
| `NODE_ENV` | Runtime | `production` | Node.js environment |

---

## 7. Multi-Architecture Building

### How It Works

**Architecture Detection**:
```bash
# On user's machine:
docker pull ghcr.io/samvellauk/dockge-plus:latest

# Docker automatically selects the correct architecture:
- Intel/AMD PC → pulls linux/amd64 image
- Apple Silicon → pulls linux/arm64 image
- Raspberry Pi → pulls linux/arm/v7 image
```

**Manifest Structure**:
```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.index.v1+json",
  "manifests": [
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "digest": "sha256:abc123...",
      "size": 1234,
      "platform": {
        "architecture": "amd64",
        "os": "linux"
      }
    },
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "digest": "sha256:def456...",
      "size": 2345,
      "platform": {
        "architecture": "arm64",
        "os": "linux"
      }
    },
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "digest": "sha256:ghi789...",
      "size": 3456,
      "platform": {
        "architecture": "arm",
        "os": "linux",
        "variant": "v7"
      }
    }
  ]
}
```

### Build Time by Architecture

**Typical build times** (GitHub Actions runners):
- `linux/amd64`: 2-3 minutes (native)
- `linux/arm64`: 8-12 minutes (QEMU emulation)
- `linux/arm/v7`: 10-15 minutes (QEMU emulation)

**Why emulation is slower**:
- Every ARM instruction is translated to x86_64
- npm install compiles native modules (slower under QEMU)
- No JIT optimization for emulated code

---

## 8. Frontend Build Process

### Build Command

```bash
npm run build:frontend
# Runs: vite build --config ./frontend/vite.config.ts
```

### Vite Configuration

**Input** (frontend/vite.config.ts):
```typescript
{
  root: "./frontend",              // Source directory
  build: {
    outDir: "../frontend-dist"     // Output to project root
  },
  plugins: [
    vue(),                         // Vue 3 SFC compilation
    Components({                   // Auto-import components
      resolvers: [BootstrapVueNextResolver()]
    }),
    viteCompression({              // gzip compression
      algorithm: "gzip",
      filter: /\.(js|mjs|json|css|html|svg)$/i
    }),
    viteCompression({              // Brotli compression
      algorithm: "brotliCompress",
      filter: /\.(js|mjs|json|css|html|svg)$/i
    })
  ]
}
```

### Output Structure

```
frontend-dist/
├── index.html                      # Entry HTML (references hashed assets)
├── assets/
│   ├── index-[hash].js            # Main JS bundle
│   ├── index-[hash].js.gz         # gzipped version
│   ├── index-[hash].js.br         # Brotli version
│   ├── index-[hash].css           # Main CSS bundle
│   ├── index-[hash].css.gz
│   ├── index-[hash].css.br
│   ├── vendor-[hash].js           # Third-party dependencies
│   ├── logo-[hash].svg            # Images (optimized)
│   └── ...
└── (other static assets)
```

### Serving in Production

**Backend code** (backend/dockge-server.ts):
```typescript
import expressStaticGzip from "express-static-gzip";

// Serves frontend-dist/ with compression
app.use(expressStaticGzip("frontend-dist", {
  enableBrotli: true,              // Prefer .br files
  orderPreference: ["br", "gz"]    // Fallback to .gz, then uncompressed
}));
```

**HTTP response**:
```
GET /assets/index-abc123.js HTTP/1.1
Host: localhost:5001

HTTP/1.1 200 OK
Content-Type: application/javascript
Content-Encoding: br               # Brotli encoding
Content-Length: 12345              # Compressed size (much smaller)
Cache-Control: public, max-age=31536000  # 1 year cache (hash in filename)
```

---

## 9. Cache Strategy

### GitHub Actions Cache

**Cache scopes**:
- Repository-specific (not shared between repos)
- Branch-specific (with fallback to default branch)
- Key format: `buildkit-{hash}`

**What's cached**:
```
type=gha,mode=max
```

- **mode=max**: Caches **all intermediate layers**, not just final image
- Includes:
  - Base image pulls (louislam/dockge:base)
  - npm ci results (node_modules layers)
  - COPY layers (if files unchanged)

**Cache benefits**:

| Scenario | Build Time (no cache) | Build Time (with cache) |
|----------|------------------------|--------------------------|
| First build | 15-20 min | N/A |
| No code changes | 15-20 min | 2-3 min |
| Backend change only | 15-20 min | 8-10 min |
| Dependency change | 15-20 min | 12-15 min |

### Docker Layer Caching

**How Docker caches layers**:

```dockerfile
# Layer 1: Base image (cached if unchanged)
FROM louislam/dockge:base AS release

# Layer 2: Healthcheck binary (cached if louislam/dockge:build-healthcheck unchanged)
COPY --from=build_healthcheck /app/extra/healthcheck /app/extra/healthcheck

# Layer 3: node_modules (cached if package-lock.json unchanged)
COPY --from=build /app/node_modules /app/node_modules

# Layer 4: Application code (invalidated if any file changes)
COPY . .

# Layer 5: mkdir data (cached if previous layers cached)
RUN mkdir ./data
```

**Cache invalidation**:
- If package-lock.json changes → layers 3, 4, 5 rebuild
- If backend code changes → only layers 4, 5 rebuild
- If base image changes → all layers rebuild

---

## 10. Common Issues & Solutions

### Issue 1: "ERROR: failed to solve: dockerfile parse error"

**Cause**: Syntax error in Dockerfile

**Solution**:
```bash
# Validate Dockerfile locally
docker build -f docker/Dockerfile . --check
```

---

### Issue 2: "ERROR: failed to push: unexpected status: 403 Forbidden"

**Cause**: Insufficient permissions to push to GHCR

**Solution**:
- Check workflow has `packages: write` permission
- Verify GITHUB_TOKEN is not expired
- Check package visibility (might be locked)

---

### Issue 3: "Frontend assets not found (404)"

**Cause**: Frontend not built before Docker build, or path mismatch

**Solution**:
- Ensure `npm run build:frontend` runs **before** Docker build in workflow
- Verify `frontend-dist/` is not in `.dockerignore`
- Check `COPY . .` in Dockerfile includes `frontend-dist/`

**Debug**:
```bash
# Run workflow steps locally
npm clean-install
npm run build:frontend
ls -la frontend-dist/  # Should show assets/

# Build Docker image
docker build -f docker/Dockerfile -t test .

# Check if frontend-dist is in image
docker run --rm test ls -la /app/frontend-dist/
```

---

### Issue 4: "QEMU binary not found or out of date"

**Cause**: QEMU not installed or outdated

**Solution**:
```yaml
- name: Set up QEMU
  uses: docker/setup-qemu-action@v3  # Must be v3+
```

---

### Issue 5: "npm ci exited with code 1"

**Cause**: package-lock.json out of sync with package.json

**Solution**:
```bash
# Regenerate package-lock.json
rm package-lock.json
npm install
git add package-lock.json
git commit -m "fix: regenerate package-lock.json"
```

---

### Issue 6: Multi-arch build times out (120 minutes)

**Cause**: ARM builds under QEMU are very slow

**Solution**:
- Increase timeout: `timeout-minutes: 240`
- Or use native ARM runners (GitHub Actions hosted or self-hosted)
- Or build fewer platforms: `platforms: linux/amd64,linux/arm64` (drop arm/v7)

---

### Issue 7: "Package not found" when pulling image

**Cause**: Package visibility is private, or doesn't exist

**Solution**:
```bash
# Check package visibility
gh api /users/SamVellaUK/packages/container/dockge-plus

# Make public manually
gh api --method PATCH \
  /users/SamVellaUK/packages/container/dockge-plus \
  -f visibility='public'
```

---

### Issue 8: Cache not working (builds always slow)

**Cause**: Cache key mismatch or storage limit exceeded

**Solution**:
- GitHub Actions cache limit: 10 GB per repository
- Old caches are auto-deleted (7 days unused)
- Check cache usage:
  ```bash
  gh api /repos/SamVellaUK/dockge-plus/actions/cache/usage
  ```

---

### Issue 9: Base image pull fails

**Cause**: `louislam/dockge:base` doesn't exist or is private

**Solution**:
```bash
# Build and push base image first
npm run build:docker-base

# Or use a public alternative
# Change in docker/Dockerfile:
FROM node:22-bookworm-slim AS base
RUN apt update && apt install --yes --no-install-recommends \
    curl ca-certificates gnupg unzip dumb-init ...
# (copy full Base.Dockerfile content)

FROM base AS release
# (rest of Dockerfile)
```

---

### Issue 10: Healthcheck always failing

**Cause**: Healthcheck binary not executable, or port mismatch

**Solution**:
```bash
# Check binary permissions in image
docker run --rm ghcr.io/samvellauk/dockge-plus:latest ls -la /app/extra/healthcheck
# Should show: -rwxr-xr-x (executable)

# Check healthcheck logs
docker inspect ghcr.io/samvellauk/dockge-plus:latest | jq '.[0].State.Health'

# Test healthcheck manually
docker run --rm ghcr.io/samvellauk/dockge-plus:latest /app/extra/healthcheck
```

---

## Summary

This Docker build system uses:

1. **Multi-stage Dockerfiles** to minimize final image size
2. **Pre-built base images** to speed up builds
3. **GitHub Actions** for automated CI/CD
4. **Docker Buildx** for multi-architecture support
5. **QEMU** for cross-compilation
6. **Vite** for optimized frontend builds
7. **Layer caching** to reduce build times
8. **GHCR** for free, unlimited public image hosting

**Key success factors**:
- Frontend built **outside Docker** (faster, better caching)
- Production dependencies installed in separate stage
- Multi-arch builds parallelized
- Comprehensive caching strategy
- Automated package visibility management

**Workflow execution time**: ~10-15 minutes (with cache), ~15-25 minutes (cold start)

---

**Generated**: 2026-02-09
**For**: dockge-plus project
**Repository**: https://github.com/SamVellaUK/dockge-plus
