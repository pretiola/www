# Pretiola Website Template

A minimal, production-ready website template built on Rust (Actix-web) with Tera templates, Tailwind CSS, and an automated Sharp image optimization pipeline. Deployed via Docker to Fly.io.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v20+)

## Quick Start

```bash
# Install Node dependencies (Tailwind, Sharp)
npm install

# Build CSS and optimize images
npm run build:css
npm run build:images

# Run the server
cargo run
# Open http://localhost:8080
```

## Adding Content

- **Pages:** Drop `.html` files into `templates/`. They auto-route at `/{filename}.html` and appear in the sitemap.
- **Images:** Add source images to `static/pictures/`, then run `npm run build:images`. The pipeline generates 3 sizes (400w/800w/1200w) in WebP + JPEG. Reference them in templates as:
  ```html
  <picture>
    <source type="image/webp"
      srcset="/pictures/optimized/{name}_1200w.webp 1200w, /pictures/optimized/{name}_800w.webp 800w, /pictures/optimized/{name}_400w.webp 400w"
      sizes="(max-width: 640px) 100vw, 800px">
    <source type="image/jpeg"
      srcset="/pictures/optimized/{name}_1200w.jpg 1200w, /pictures/optimized/{name}_800w.jpg 800w, /pictures/optimized/{name}_400w.jpg 400w"
      sizes="(max-width: 640px) 100vw, 800px">
    <img src="/pictures/optimized/{name}_800w.jpg" alt="Description">
  </picture>
  ```
- **Styles:** Edit `static/css/styles.css` (Tailwind v4), then `npm run build:css`.

## Testing

```bash
cargo test
```

## Deploy

```bash
# Docker (local)
docker compose up --build

# Fly.io
# 1. Update app name in fly.toml
# 2. Set FLY_API_TOKEN in GitHub secrets
# 3. Push to main — CI handles the rest
```

## File Structure

```
├── src/                    # Rust server source
│   ├── main.rs             # Entry point (port 8080)
│   ├── lib.rs              # Module exports
│   ├── startup.rs          # App config, route registration
│   └── routes.rs           # Request handlers + sitemap
├── templates/              # Tera HTML templates (auto-routed)
│   ├── index.html          # Home page
│   ├── terms.html          # Terms of use
│   ├── privacy.html        # Privacy policy
│   ├── navbar.html         # Navigation partial
│   └── footer.html         # Footer partial
├── static/
│   ├── css/styles.css      # Tailwind source
│   ├── js/gallery.js       # Gallery carousel + lightbox
│   └── pictures/           # Source images (optimized/ is generated)
├── scripts/
│   └── optimize-images.js  # Sharp image pipeline
├── tests/
│   └── health_check.rs     # Integration tests
├── Dockerfile              # Multi-stage build
├── docker-compose.yml      # Local Docker dev
├── fly.toml                # Fly.io config
└── .github/workflows/      # CI/CD
```
