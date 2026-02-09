# Deployment Guide

This guide covers deploying the Eclipse web interface to various platforms.

## Vercel (Recommended)

Vercel is the easiest way to deploy the Eclipse web interface.

### Prerequisites

1. WASM files must be built and committed to the repository
2. Your repository must be pushed to GitHub/GitLab/Bitbucket

### Initial Setup

#### 1. Build WASM Module

From the **project root** (not the web directory):

```bash
# Build WASM
wasm-pack build --target web --no-default-features --features wasm

# Copy to web/src
cp -r pkg web/src/

# Commit the WASM files
cd web
git add src/pkg/
git commit -m "Add built WASM files for deployment"
git push
```

#### 2. Deploy to Vercel

**Option A: Using Vercel Dashboard**

1. Go to [vercel.com](https://vercel.com) and sign in
2. Click "Add New Project"
3. Import your repository
4. Configure the project:
   - **Framework Preset**: Astro
   - **Root Directory**: `web`
   - **Build Command**: `bash build.sh` (or leave default)
   - **Output Directory**: `dist`
5. Click "Deploy"

**Option B: Using Vercel CLI**

```bash
# Install Vercel CLI
npm i -g vercel

# From the web directory
cd web

# Deploy
vercel

# Follow the prompts:
# - Link to existing project or create new
# - Confirm settings
```

### Updating the Site

When you make changes to the Rust code:

```bash
# From project root
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg web/src/

# Commit and push
cd web
git add src/pkg/
git commit -m "Update WASM module"
git push

# Vercel will automatically redeploy
```

When you make changes to the web interface only (no Rust changes):

```bash
# Just commit and push from web directory
cd web
git add .
git commit -m "Update web interface"
git push

# Vercel will automatically redeploy
```

### Configuration

The `vercel.json` file is already configured:

```json
{
  "buildCommand": "bash build.sh",
  "outputDirectory": "dist",
  "installCommand": "pnpm install",
  "framework": "astro"
}
```

## Netlify

### Initial Setup

1. Build WASM (see Vercel section above)
2. Commit WASM files to repository
3. Go to [netlify.com](https://www.netlify.com) and sign in
4. Click "Add new site" → "Import an existing project"
5. Configure:
   - **Base directory**: `web`
   - **Build command**: `pnpm run build`
   - **Publish directory**: `web/dist`
6. Click "Deploy site"

### Netlify Configuration File

Create `web/netlify.toml`:

```toml
[build]
  base = "web"
  command = "pnpm run build"
  publish = "dist"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

## GitHub Pages

### Setup

1. Build WASM and commit (see above)
2. In your repository, go to Settings → Pages
3. Set source to "GitHub Actions"
4. Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [ master ]
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Install pnpm
        run: npm install -g pnpm

      - name: Install dependencies
        working-directory: ./web
        run: pnpm install

      - name: Build
        working-directory: ./web
        run: pnpm run build

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v2
        with:
          path: ./web/dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v2
```

5. Update `web/astro.config.mjs` with your site URL:

```javascript
export default defineConfig({
  site: 'https://yourusername.github.io',
  base: '/your-repo-name',
  // ... rest of config
});
```

## Cloudflare Pages

1. Build WASM and commit (see above)
2. Go to Cloudflare Pages dashboard
3. Create a new project
4. Configure:
   - **Framework**: Astro
   - **Build command**: `cd web && pnpm run build`
   - **Build output directory**: `web/dist`
   - **Root directory**: Leave empty (or set to `web`)

## Docker

Create `web/Dockerfile`:

```dockerfile
FROM node:18-alpine AS builder

WORKDIR /app

# Copy web directory
COPY . .

# Install dependencies and build
RUN npm install -g pnpm && \
    pnpm install && \
    pnpm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

Build and run:

```bash
cd web
docker build -t eclipse-web .
docker run -p 8080:80 eclipse-web
```

## Troubleshooting

### WASM files not found during build

**Error**: `Could not resolve "../pkg/eclipse.js"`

**Solution**: Make sure WASM files are built and committed:

```bash
# From project root
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg web/src/
cd web
git add src/pkg/
git commit -m "Add WASM files"
```

### Build timeout on deployment platform

**Problem**: Building Rust during deployment takes too long

**Solution**: This is why we commit pre-built WASM files. Don't try to build Rust on the deployment platform.

### Wrong root directory

**Error**: Can't find package.json

**Solution**: Make sure the root directory is set to `web` in your deployment platform settings.

### Module not found errors

**Solution**: Ensure all dependencies are in `package.json` and you're using the correct Node version (18+).

## Performance Optimization

For production builds, the WASM module is already optimized by `wasm-pack`. The Astro build automatically:

- Minifies JavaScript
- Optimizes CSS
- Compresses assets
- Generates static HTML

No additional configuration needed!

## Custom Domain

Most platforms make it easy to add a custom domain:

- **Vercel**: Domains → Add Domain
- **Netlify**: Domain settings → Add custom domain
- **Cloudflare Pages**: Custom domains → Set up a custom domain

## Monitoring

After deployment, you can monitor:

- **Build logs**: Check for warnings or errors
- **Performance**: Use Lighthouse or WebPageTest
- **WASM loading**: Check browser console for initialization messages

The WASM module logs "WASM module initialized successfully" when ready.
