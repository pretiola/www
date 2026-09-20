# Local MHRI distribution

Source: https://github.com/headertag/mhri
Commit: 44ab49649b40d45d3e5f56bf6d4c41d696ebcda7

Built with npm ci and npm run build -- --base=/static/mhri/.
Before building, absolute /art/ and /fonts/ references in src and index.html
were prefixed with /static/mhri/ to isolate the embedded application's assets.
The resulting dist directory is checked in here so ordinary Pretiola builds
need no additional dependency installation. Upstream README and third-party
notices are retained alongside the assets; font licenses are in fonts/.

The parent page provides an original-painting fallback and descriptive link
without JavaScript. The interactive scene is isolated in a same-origin iframe.

The built MHRI home link is redirected to /static/mhri/index.html to avoid nesting the Pretiola homepage inside the frame.
