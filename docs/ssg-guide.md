# Static Site Generation (SSG) Guide

## Overview

Virust v0.5 introduces powerful Static Site Generation (SSG) and Incremental Static Regeneration (ISR) capabilities, allowing you to pre-render pages at build time for optimal performance while maintaining the flexibility of server-side rendering.

### Key Benefits

- **Lightning-fast page loads** - Pre-rendered HTML served instantly
- **Better SEO** - Fully rendered HTML available to crawlers
- **Reduced server load** - Static files served directly from disk
- **CDN-friendly** - Deploy static files anywhere
- **ISR support** - Update content without full rebuilds
- **Hybrid rendering** - Mix SSG, SSR, and API routes in one app

## Quick Start

### 1. Create an SSG-enabled Route

Add the `#[ssg]` attribute to any route handler:

```rust
// api/route.rs
use virust_macros::{get, ssg};
use virust_runtime::RenderedHtml;

#[get]
#[ssg]
pub async fn index() -> RenderedHtml {
    RenderedHtml::new("HomePage")
}
```

### 2. Build Your Static Site

```bash
# Build static site (outputs to dist/ by default)
virust build --ssg

# Specify custom output directory
virust build --ssg --output out/

# Control parallelism (default: number of CPU cores)
virust build --ssg --jobs 8
```

### 3. Serve Your Static Site

```bash
# Serve the built static site
virust serve --static dist/

# Or use any static file server
npx serve dist/
```

The build process will:
- Discover all routes with `#[ssg]` attribute
- Render each route to static HTML
- Generate ISR metadata for routes with revalidation
- Output files to the specified directory

## Understanding SSG vs SSR vs ISR

### Static Site Generation (SSG)

**When to use:**
- Blog posts, documentation
- Marketing pages, landing pages
- Content that doesn't change frequently
- Public pages that need SEO

**How it works:**
```rust
#[get]
#[ssg]  // No revalidation = fully static
pub async fn about_page() -> RenderedHtml {
    RenderedHtml::new("AboutPage")
}
```

**Pros:**
- Fastest page loads (no server processing)
- Can be deployed to CDNs
- Minimal server costs
- Best SEO (content always available)

**Cons:**
- Must rebuild to update content
- Not suitable for personalized content
- Build time scales with page count

### Server-Side Rendering (SSR)

**When to use:**
- User-specific data (dashboards, profiles)
- Real-time data (stock prices, live stats)
- Authentication-required pages
- Highly dynamic content

**How it works:**
```rust
#[get]
#[render_component("Dashboard")]
pub async fn dashboard() -> RenderedHtml {
    // Renders on every request
    RenderedHtml::new("Dashboard")
}
```

**Pros:**
- Always shows latest data
- Can personalize content
- No rebuild needed

**Cons:**
- Slower than static (server processing)
- Higher server costs
- Can't deploy to CDN

### Incremental Static Regeneration (ISR)

**When to use:**
- Content that updates periodically (blogs, news)
- Pages with mostly static content
- When you want fast loads + fresh content
- To reduce build times while staying current

**How it works:**
```rust
#[get]
#[ssg(revalidate = 3600)]  // Revalidate every hour
pub async fn blog_post(#[path] slug: String) -> RenderedHtml {
    RenderedHtml::new("BlogPost")
}
```

**Pros:**
- Fast static page loads
- Content stays fresh
- Background regeneration
- No full rebuilds needed

**Cons:**
- Slightly more complex than pure SSG
- Content can be stale until revalidation
- Requires server for revalidation

## Using the `#[ssg]` Attribute

### Basic Static Generation

```rust
use virust_macros::{get, ssg};
use virust_runtime::RenderedHtml;

#[get]
#[ssg]
pub async fn home() -> RenderedHtml {
    // This page will be built once and served as static HTML
    RenderedHtml::new("HomePage")
}

#[get]
#[ssg]
pub async fn pricing() -> RenderedHtml {
    // Another static page
    RenderedHtml::new("PricingPage")
}
```

### Dynamic Routes with SSG

```rust
// api/blog/[slug]/route.rs
#[get]
#[ssg(revalidate = 3600)]  // ISR: revalidate every hour
pub async fn blog_post(#[path] slug: String) -> RenderedHtml {
    // Each blog post will be pre-rendered
    // and regenerated after 1 hour
    RenderedHtml::new("BlogPost")
}
```

### Nested Dynamic Routes

```rust
// api/docs/[version]/[section]/route.rs
#[get]
#[ssg]
pub async fn docs_page(
    #[path] version: String,
    #[path] section: String
) -> RenderedHtml {
    // Will generate static pages for each combination
    // e.g., /docs/v0.4/getting-started, /docs/v0.5/api
    RenderedHtml::new("DocsPage")
}
```

## ISR: Incremental Static Regeneration

### How ISR Works

ISR gives you the best of both worlds:
1. **Build time** - Pages are pre-rendered to static HTML
2. **Request time** - Static HTML is served instantly
3. **Background** - After `revalidate` seconds, page is regenerated
4. **Next request** - Fresh static HTML is served

### Setting Revalidation Times

```rust
#[get]
#[ssg(revalidate = 60)]      // Revalidate every minute
pub async fn frequently_updated() -> RenderedHtml {
    RenderedHtml::new("FrequentlyUpdated")
}

#[get]
#[ssg(revalidate = 3600)]    // Revalidate every hour
pub async fn hourly_updates() -> RenderedHtml {
    RenderedHtml::new("HourlyUpdates")
}

#[get]
#[ssg(revalidate = 86400)]   // Revalidate daily
pub async fn daily_updates() -> RenderedHtml {
    RenderedHtml::new("DailyUpdates")
}
```

### Revalidation Timing Guidelines

| Content Type | Revalidate Time | Example |
|--------------|----------------|---------|
| Real-time data | 60-300 seconds | Stock prices, scores |
| News articles | 300-3600 seconds | News feed, announcements |
| Blog posts | 3600-86400 seconds | Blog, documentation |
| Marketing pages | None (static) | About, pricing, contact |

### ISR Metadata

When you build with ISR, Virust generates `.isr-metadata.json` in your output directory:

```json
{
  "routes": {
    "/blog/hello-world": {
      "path": "/blog/hello-world",
      "file_path": "blog/hello-world/index.html",
      "generated_at": "2026-03-06T12:00:00Z",
      "revalidate": 3600,
      "tags": ["blog"]
    }
  },
  "generated_at": "2026-03-06T12:00:00Z"
}
```

This metadata is used by the runtime to:
- Track when pages were last generated
- Trigger background revalidation
- Serve stale-while-revalidate content

## Caching with `#[cache]` Attribute

The `#[cache]` attribute adds HTTP caching to your API routes:

### Basic Caching

```rust
use virust_macros::{get, cache};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    message: String,
}

#[get]
#[cache(max_age = 300)]  // Cache for 5 minutes
pub async fn get_data() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Cached data".to_string(),
    })
}
```

### Cache Duration Guidelines

| Data Type | max_age | Description |
|-----------|---------|-------------|
| Static data | 3600-86400 | Configuration, reference data |
| User profiles | 300-3600 | Public profile information |
| API responses | 60-300 | Frequently accessed data |
| Real-time data | 0-60 | Stock prices, live updates |

### Cache Headers

The `#[cache]` attribute automatically adds appropriate HTTP headers:

```
Cache-Control: public, max-age=300
ETag: "abc123"
X-Cache: HIT
```

### Combining `#[cache]` with ISR

```rust
#[get]
#[ssg(revalidate = 3600)]  // ISR: regenerate every hour
#[cache(max_age = 300)]    // HTTP cache: 5 minutes
pub async fn blog_post(#[path] slug: String) -> RenderedHtml {
    RenderedHtml::new("BlogPost")
}
```

This gives you:
- **5-minute browser cache** - Reduces repeated requests
- **1-hour ISR** - Background regeneration
- **Instant static serving** - Pre-rendered HTML

## Advanced Topics

### Parallel Builds

Virust automatically builds pages in parallel using all available CPU cores:

```bash
# Use 4 parallel jobs (default: number of cores)
virust build --ssg --jobs 4

# Use more jobs for faster builds (uses more memory)
virust build --ssg --jobs 16
```

**Performance tips:**
- More jobs = faster builds (up to a point)
- Each job uses ~50-100MB memory
- Sweet spot: 2-4x your CPU core count for I/O-bound builds

### Build Statistics

After building, you'll see statistics:

```
✓ Build complete!
  Pages built: 150
  Pages failed: 0
  Routes with ISR: 50
  Build time: 2.3s
  Output: dist/
```

### Hybrid Rendering Strategy

You can mix SSG, SSR, and API routes in the same application:

```rust
// Static pages (build once, serve forever)
#[get]
#[ssg]
pub async fn about() -> RenderedHtml {
    RenderedHtml::new("AboutPage")
}

// ISR pages (build + regenerate)
#[get]
#[ssg(revalidate = 3600)]
pub async fn blog_post(#[path] slug: String) -> RenderedHtml {
    RenderedHtml::new("BlogPost")
}

// SSR pages (render on every request)
#[get]
#[render_component("Dashboard")]
pub async fn dashboard() -> RenderedHtml {
    RenderedHtml::new("Dashboard")
}

// API routes (no rendering)
#[get]
#[cache(max_age = 300)]
pub async fn api_data() -> Json<Data> {
    // ...
}
```

### Excluding Routes from SSG

Routes without `#[ssg]` are not included in the static build:

```rust
#[get]
// No #[ssg] attribute - this won't be built statically
pub async fn admin_panel() -> RenderedHtml {
    RenderedHtml::new("AdminPanel")
}
```

### Static File Serving

After building, serve your static files:

```rust
use virust_runtime::isr::IsrManager;
use axum::Router;

#[tokio::main]
async fn main() {
    let isr_manager = IsrManager::new("dist/".into()).unwrap();

    let app = Router::new()
        .nest_service("/static", axum::routing::get_service(
            ServeDir::new("dist")
        ))
        .fallback(isr_serve);

    // ... start server
}
```

## Real-World Examples

### Blog with ISR

```rust
// api/route.rs
#[get]
#[ssg]
pub async fn index() -> RenderedHtml {
    RenderedHtml::new("BlogIndex")
}

// api/blog/[slug]/route.rs
#[get]
#[ssg(revalidate = 3600)]
pub async fn blog_post(#[path] slug: String) -> RenderedHtml {
    // Fetch blog post data
    let post = fetch_post(&slug).await;

    // Pass to component via props
    RenderedHtml::with_props("BlogPost", post)
}
```

### E-commerce Product Pages

```rust
// api/products/[id]/route.rs
#[get]
#[ssg(revalidate = 300)]  // Revalidate every 5 minutes
#[cache(max_age = 60)]     // Browser cache 1 minute
pub async fn product_page(#[path] id: String) -> RenderedHtml {
    let product = fetch_product(&id).await;
    RenderedHtml::with_props("ProductPage", product)
}

// api/products/route.rs
#[get]
#[ssg]  // Build product listing once
pub async fn product_list() -> RenderedHtml {
    let products = fetch_all_products().await;
    RenderedHtml::with_props("ProductList", products)
}
```

### Documentation Site

```rust
// api/route.rs
#[get]
#[ssg]
pub async fn docs_index() -> RenderedHtml {
    RenderedHtml::new("DocsIndex")
}

// api/docs/[version]/[section]/route.rs
#[get]
#[ssg]  // Fully static documentation
pub async fn docs_page(
    #[path] version: String,
    #[path] section: String
) -> RenderedHtml {
    let content = load_docs(&version, &section).await;
    RenderedHtml::with_props("DocsPage", content)
}
```

### Hybrid App with Multiple Strategies

```rust
// Public marketing pages - fully static
#[get]
#[ssg]
pub async fn home() -> RenderedHtml {
    RenderedHtml::new("HomePage")
}

#[get]
#[ssg]
pub async fn pricing() -> RenderedHtml {
    RenderedHtml::new("PricingPage")
}

// Blog with ISR
#[get]
#[ssg(revalidate = 3600)]
pub async fn blog(#[path] slug: String) -> RenderedHtml {
    RenderedHtml::new("BlogPost")
}

// User dashboard - SSR
#[get]
#[render_component("Dashboard")]
pub async fn dashboard() -> RenderedHtml {
    // Rendered on every request with user data
    RenderedHtml::new("Dashboard")
}

// Admin panel - SSR, no caching
#[get]
pub async fn admin() -> RenderedHtml {
    RenderedHtml::new("AdminPanel")
}
```

## Deployment

### Deploy to Static Hosting

```bash
# Build static site
virust build --ssg --output dist/

# Deploy to Netlify
netlify deploy --prod --dir=dist

# Deploy to Vercel
vercel --prod dist

# Deploy to GitHub Pages
gh-pages -d dist
```

### Deploy with ISR Support

For ISR, you need a server:

```bash
# Build static site
virust build --ssg

# Build server binary
virust build --release

# Run server (serves static + handles ISR)
./target/release/virust-server
```

### CI/CD Pipeline

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Virust
        run: curl -fsSL https://raw.githubusercontent.com/hangsiahong/virust/master/install.sh | bash

      - name: Build Static Site
        run: virust build --ssg --output dist/

      - name: Deploy to Netlify
        run: npx netlify-cli deploy --prod --dir=dist
```

## Performance Tips

1. **Use SSG for content that doesn't change per-user**
   - Marketing pages, blog posts, documentation

2. **Use ISR for content that updates periodically**
   - News feeds, product listings, dashboards

3. **Use SSR for user-specific or real-time data**
   - User profiles, admin panels, live data

4. **Cache API responses**
   - Use `#[cache]` on frequently accessed API endpoints

5. **Optimize parallel builds**
   - Use `--jobs` flag to match your system

6. **Monitor build times**
   - Build stats show pages built/failed and total time

7. **Use CDN for static assets**
   - Deploy `dist/` to CDN for global performance

## Troubleshooting

### Build Fails

**Problem:** "No SSG routes found"
```bash
# Solution: Make sure routes have #[ssg] attribute
#[get]
#[ssg]  # ← This is required!
pub async fn page() -> RenderedHtml { }
```

**Problem:** Build is slow
```bash
# Solution: Increase parallel jobs (careful with memory)
virust build --ssg --jobs 16
```

### ISR Not Regenerating

**Problem:** Stale content not updating
```bash
# Solution: Check server logs for revalidation errors
# Solution: Verify .isr-metadata.json exists in output dir
# Solution: Ensure server has write permissions for revalidation
```

### Caching Issues

**Problem:** Old content served from cache
```bash
# Solution: Reduce max_age time
#[cache(max_age = 60)]  # 1 minute instead of 1 hour
```

**Problem:** Cache not working
```bash
# Solution: Ensure route has #[get] attribute
#[get]  # ← Required for caching
#[cache(max_age = 300)]
pub async fn data() -> Json<Data> { }
```

## Next Steps

- Check out the [SSR Guide](./ssr-guide.md) for server-side rendering
- See [VIRUST_VS_NEXTJS.md](./VIRUST_VS_NEXTJS.md) for comparisons
- Explore templates with `virust init -t ssr-blog`
- Read [v0.5 Release Notes](./plans/2026-03-06-virust-v0.5-release.md)

## Summary

Virust v0.5 SSG gives you:

- **Static Site Generation** - Pre-render pages at build time
- **Incremental Static Regeneration** - Update pages without full rebuilds
- **HTTP Caching** - Add cache control to API routes
- **Parallel Builds** - Fast builds using multiple CPU cores
- **Hybrid Rendering** - Mix SSG, ISR, and SSR in one app

Start with `#[ssg]` for static pages, add `revalidate` for ISR, and use `#[cache]` for API routes. Happy building!
