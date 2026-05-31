# VTP Node Deployment Guide

<div align="center">

**Production deployment instructions for VTP Node**

[Overview](#overview) • [Deployment Options](#deployment-options) • [Build Process](#build-process) • [Hosting Platforms](#hosting-platforms) • [CI/CD](#cicd) • [Monitoring](#monitoring)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Deployment Options](#deployment-options)
  - [Static Hosting](#static-hosting)
  - [Docker](#docker)
  - [Self-Hosted](#self-hosted)
- [Build Process](#build-process)
  - [Production Build](#production-build)
  - [Environment Variables](#environment-variables)
  - [Build Optimization](#build-optimization)
- [Hosting Platforms](#hosting-platforms)
  - [Cloudflare Pages](#cloudflare-pages)
  - [Vercel](#vercel)
  - [Netlify](#netlify)
  - [GitHub Pages](#github-pages)
  - [AWS S3 + CloudFront](#aws-s3--cloudfront)
- [CI/CD](#cicd)
  - [GitHub Actions](#github-actions)
  - [GitLab CI](#gitlab-ci)
  - [Jenkins](#jenkins)
- [Monitoring](#monitoring)
  - [Performance Monitoring](#performance-monitoring)
  - [Error Tracking](#error-tracking)
  - [Analytics](#analytics)
- [Security](#security)
  - [HTTPS](#https)
  - [Content Security Policy](#content-security-policy)
  - [CORS](#cors)
- [Troubleshooting](#troubleshooting)

---

## Overview

VTP Node is a static web application that can be deployed to any static hosting platform. The application consists of:

1. **HTML/CSS/JavaScript**: Svelte application compiled to static files
2. **WebAssembly**: Rust core library compiled to Wasm
3. **Web Worker**: Background computation thread
4. **Service Worker**: PWA caching layer

### Deployment Requirements

- **HTTPS**: Required for Service Workers and PWA
- **Static File Serving**: All files must be served as static assets
- **Wasm MIME Type**: `.wasm` files must be served with `application/wasm` content type
- **Caching**: Proper cache headers for static assets

---

## Deployment Options

### Static Hosting

The simplest deployment option. Upload the `build/` directory to any static hosting provider.

**Pros:**

- Simple setup
- Low cost
- High availability
- Global CDN

**Cons:**

- No server-side logic
- Limited customization

### Docker

Deploy using Docker for more control over the environment.

**Pros:**

- Consistent environment
- Easy scaling
- Custom configuration

**Cons:**

- More complex setup
- Higher resource usage

### Self-Hosted

Host on your own server with Nginx or Apache.

**Pros:**

- Full control
- Custom domain
- No vendor lock-in

**Cons:**

- Maintenance overhead
- Security responsibility

---

## Build Process

### Production Build

Run the production build command:

```bash
# Clean previous builds
rm -rf build/

# Install dependencies
npm ci

# Build WebAssembly
npm run wasm:build

# Build Worker
npm run worker:build

# Build application
NODE_ENV=production npm run build
```

The output will be in the `build/` directory.

### Environment Variables

Create a `.env.production` file:

```env
# Application
NODE_ENV=production
BASE_URL=https://vtp-node.dev

# VDF Configuration
VDF_DEFAULT_TOTAL=1000000
VDF_CHECKPOINT_INTERVAL=100000

# Performance
WORKER_BATCH_SIZE=1000
WORKER_TIME_SLICE_MS=50

# Analytics (optional)
GA_TRACKING_ID=UA-XXXXXXXXX-X
```

### Build Optimization

#### Rust/Wasm Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
strip = true         # Strip debug symbols
```

#### JavaScript Optimization

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true
      }
    },
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['svelte']
        }
      }
    }
  }
});
```

#### Asset Optimization

```bash
# Optimize images
npm run optimize:images

# Compress Wasm
npm run compress:wasm

# Generate gzip files
npm run compress:gzip
```

---

## Hosting Platforms

### Cloudflare Pages

**Recommended for VTP Node**

#### Setup

1. **Connect Repository**
   - Go to Cloudflare Dashboard
   - Navigate to Pages
   - Click "Create a project"
   - Connect your GitHub/GitLab repository

2. **Configure Build**
   - Build command: `npm run build`
   - Build output directory: `build`
   - Node.js version: 18

3. **Environment Variables**

   ```
   NODE_ENV=production
   ```

4. **Deploy**
   - Push to main branch
   - Cloudflare will automatically build and deploy

#### Custom Domain

1. Go to Pages project settings
2. Click "Custom domains"
3. Add your domain
4. Configure DNS records

#### Headers

Create `public/_headers` file:

```
/*
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: camera=(), microphone=(), geolocation=()

/*.wasm
  Content-Type: application/wasm

/sw.js
  Cache-Control: no-cache

/icons/*
  Cache-Control: public, max-age=31536000, immutable
```

---

### Vercel

#### Setup

1. **Install Vercel CLI**

   ```bash
   npm i -g vercel
   ```

2. **Deploy**

   ```bash
   vercel
   ```

3. **Configure**
   - Framework Preset: SvelteKit
   - Build Command: `npm run build`
   - Output Directory: `build`

#### Configuration

Create `vercel.json`:

```json
{
  "buildCommand": "npm run build",
  "outputDirectory": "build",
  "framework": "sveltekit",
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "X-Content-Type-Options",
          "value": "nosniff"
        },
        {
          "key": "X-Frame-Options",
          "value": "DENY"
        }
      ]
    },
    {
      "source": "/(.*.wasm)",
      "headers": [
        {
          "key": "Content-Type",
          "value": "application/wasm"
        }
      ]
    }
  ]
}
```

---

### Netlify

#### Setup

1. **Connect Repository**
   - Go to Netlify Dashboard
   - Click "New site from Git"
   - Connect your repository

2. **Configure Build**
   - Build command: `npm run build`
   - Publish directory: `build`

3. **Environment Variables**
   ```
   NODE_ENV=production
   ```

#### Configuration

Create `netlify.toml`:

```toml
[build]
  command = "npm run build"
  publish = "build"

[build.environment]
  NODE_ENV = "production"

[[headers]]
  for = "/*"
  [headers.values]
    X-Frame-Options = "DENY"
    X-Content-Type-Options = "nosniff"
    Referrer-Policy = "strict-origin-when-cross-origin"

[[headers]]
  for = "*.wasm"
  [headers.values]
    Content-Type = "application/wasm"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

---

### GitHub Pages

#### Setup

1. **Enable GitHub Pages**
   - Go to repository Settings
   - Navigate to Pages
   - Select source branch (e.g., `gh-pages`)

2. **Configure GitHub Actions**

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Install dependencies
        run: npm ci

      - name: Build
        run: |
          npm run wasm:build
          npm run build

      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./build
```

---

### AWS S3 + CloudFront

#### Setup

1. **Create S3 Bucket**

   ```bash
   aws s3 mb s3://vtp-node-app
   ```

2. **Enable Static Website Hosting**

   ```bash
   aws s3 website s3://vtp-node-app \
     --index-document index.html \
     --error-document index.html
   ```

3. **Upload Files**

   ```bash
   aws s3 sync build/ s3://vtp-node-app --delete
   ```

4. **Create CloudFront Distribution**
   ```bash
   aws cloudfront create-distribution \
     --origin-domain-name vtp-node-app.s3.amazonaws.com \
     --default-root-object index.html
   ```

#### CloudFront Configuration

```json
{
  "DefaultCacheBehavior": {
    "ViewerProtocolPolicy": "redirect-to-https",
    "Compress": true,
    "CachePolicyId": "658327ea-f89d-4fab-a63d-7e88639e58f6"
  },
  "CacheBehaviors": [
    {
      "PathPattern": "*.wasm",
      "ViewerProtocolPolicy": "redirect-to-https",
      "CachePolicyId": "658327ea-f89d-4fab-a63d-7e88639e58f6",
      "ResponseHeadersPolicyId": "wasm-headers"
    }
  ]
}
```

---

## CI/CD

### GitHub Actions

Complete CI/CD pipeline:

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install dependencies
        run: npm ci

      - name: Run linting
        run: npm run lint

      - name: Run tests
        run: |
          npm run wasm:test
          npm test

      - name: Build
        run: |
          npm run wasm:build
          npm run build

  deploy:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install dependencies
        run: npm ci

      - name: Build
        run: |
          npm run wasm:build
          npm run build

      - name: Deploy to Cloudflare Pages
        uses: cloudflare/pages-action@v1
        with:
          apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          projectName: vtp-node
          directory: build
```

### GitLab CI

```yaml
stages:
  - test
  - build
  - deploy

test:
  stage: test
  image: node:18
  script:
    - npm ci
    - npm run lint
    - npm test
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - node_modules/

build:
  stage: build
  image: rust:latest
  before_script:
    - rustup target add wasm32-unknown-unknown
    - cargo install wasm-pack
  script:
    - npm ci
    - npm run wasm:build
    - npm run build
  artifacts:
    paths:
      - build/
  cache:
    key: ${CI_COMMIT_REF_SLUG}
    paths:
      - target/
      - node_modules/

deploy:
  stage: deploy
  image: node:18
  script:
    - npm install -g wrangler
    - wrangler pages deploy build --project-name=vtp-node
  only:
    - main
  environment:
    name: production
```

### Jenkins

```groovy
pipeline {
    agent any

    environment {
        NODE_VERSION = '18'
        RUST_VERSION = 'stable'
    }

    stages {
        stage('Setup') {
            steps {
                sh 'npm ci'
                sh 'rustup target add wasm32-unknown-unknown'
                sh 'cargo install wasm-pack'
            }
        }

        stage('Test') {
            steps {
                sh 'npm run lint'
                sh 'npm run wasm:test'
                sh 'npm test'
            }
        }

        stage('Build') {
            steps {
                sh 'npm run wasm:build'
                sh 'npm run build'
            }
        }

        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                sh 'aws s3 sync build/ s3://vtp-node-app --delete'
                sh 'aws cloudfront create-invalidation --distribution-id EXXXXXXXXX --paths "/*"'
            }
        }
    }

    post {
        always {
            cleanWs()
        }
    }
}
```

---

## Monitoring

### Performance Monitoring

#### Lighthouse CI

```yaml
# .github/workflows/lighthouse.yml
name: Lighthouse CI

on:
  pull_request:
    branches: [main]

jobs:
  lighthouse:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run Lighthouse CI
        uses: treosh/lighthouse-ci-action@v9
        with:
          urls: |
            http://localhost:5173/
          uploadArtifacts: true
```

#### Web Vitals

```typescript
// src/utils/analytics.ts
import { onCLS, onFID, onLCP } from 'web-vitals';

export function reportWebVitals() {
  onCLS(console.log);
  onFID(console.log);
  onLCP(console.log);
}
```

### Error Tracking

#### Sentry Integration

```typescript
// src/utils/sentry.ts
import * as Sentry from '@sentry/svelte';

Sentry.init({
  dsn: 'https://your-dsn@sentry.io/your-project',
  environment: import.meta.env.MODE,
  tracesSampleRate: 1.0
});
```

#### Error Boundary

```svelte
<!-- src/components/ErrorBoundary.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';

  let error: Error | null = null;

  onMount(() => {
    window.onerror = (message, source, lineno, colno, err) => {
      error = err || new Error(message as string);
    };
  });
</script>

{#if error}
  <div class="error-boundary">
    <h2>Something went wrong</h2>
    <p>{error.message}</p>
    <button on:click={() => window.location.reload()}> Reload Page </button>
  </div>
{:else}
  <slot />
{/if}
```

### Analytics

#### Google Analytics

```typescript
// src/utils/analytics.ts
export function initGA(trackingId: string) {
  const script = document.createElement('script');
  script.async = true;
  script.src = `https://www.googletagmanager.com/gtag/js?id=${trackingId}`;
  document.head.appendChild(script);

  window.dataLayer = window.dataLayer || [];
  function gtag(...args: any[]) {
    window.dataLayer.push(args);
  }
  gtag('js', new Date());
  gtag('config', trackingId);
}

export function trackEvent(action: string, category: string, label?: string, value?: number) {
  window.gtag?.('event', action, {
    event_category: category,
    event_label: label,
    value: value
  });
}
```

---

## Security

### HTTPS

HTTPS is required for:

- Service Workers
- PWA installation
- WebAssembly
- Secure context APIs

#### Certificate Setup

```bash
# Using Let's Encrypt
sudo apt install certbot
sudo certbot certonly --standalone -d vtp-node.dev

# Using Cloudflare
# Enable "Always Use HTTPS" in SSL/TLS settings
```

### Content Security Policy

Add CSP headers to prevent XSS attacks:

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'unsafe-inline' 'unsafe-eval';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https:;
  font-src 'self';
  connect-src 'self' https://api.vtp-node.dev;
  worker-src 'self' blob:;
  wasm-src 'self';
```

### CORS

Configure CORS for API endpoints:

```nginx
# nginx.conf
location /api/ {
    add_header 'Access-Control-Allow-Origin' 'https://vtp-node.dev';
    add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
    add_header 'Access-Control-Allow-Headers' 'Content-Type';
}
```

---

## Troubleshooting

### Common Issues

#### Wasm Not Loading

**Error**: `Failed to execute 'compile' on 'WebAssembly': Incorrect response MIME type`

**Solution**:

```nginx
# nginx.conf
location ~* \.wasm$ {
    types { application/wasm wasm; }
    add_header Cache-Control "public, max-age=31536000, immutable";
}
```

#### Service Worker Not Registering

**Error**: `Service worker registration failed`

**Solution**:

1. Ensure HTTPS is enabled
2. Check service worker file exists
3. Verify file permissions
4. Check browser console for errors

#### PWA Not Installing

**Error**: Install prompt not appearing

**Solution**:

1. Verify manifest.json is valid
2. Ensure all required icons exist
3. Check service worker is registered
4. Verify HTTPS is enabled

#### Performance Issues

**Error**: Application is slow in production

**Solution**:

1. Enable gzip compression
2. Configure caching headers
3. Optimize images
4. Use CDN

### Debug Mode

Enable debug mode in production:

```typescript
// src/config.ts
export const DEBUG =
  import.meta.env.DEV || new URLSearchParams(window.location.search).has('debug');
```

### Logs

Check application logs:

```bash
# Browser console
# Open DevTools (F12) > Console

# Service Worker logs
# Open DevTools (F12) > Application > Service Workers

# Network requests
# Open DevTools (F12) > Network
```

---

## Support

For deployment questions:

- **GitHub Issues**: [Create an issue](https://github.com/your-org/vtp-node/issues)
- **Email**: deploy@vtp-node.dev

---

<div align="center">

**[⬆ Back to Top](#vtp-node-deployment-guide)**

</div>
