const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const source = path.join(__dirname, '../static/pictures/pretzel.png');
const outputDir = path.join(__dirname, '../static');

const sizes = [
    { name: 'favicon-32x32.png', size: 32 },
    { name: 'favicon-16x16.png', size: 16 },
    { name: 'apple-touch-icon.png', size: 180 },
    { name: 'android-chrome-192x192.png', size: 192 },
    { name: 'android-chrome-512x512.png', size: 512 },
];

async function generateFavicons() {
    for (const { name, size } of sizes) {
        await sharp(source)
            .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
            .png({ quality: 90, compressionLevel: 9 })
            .toFile(path.join(outputDir, name));
        console.log(`  -> ${name} (${size}x${size})`);
    }

    // Generate ICO-compatible 32x32 PNG as favicon.ico
    // Sharp doesn't output .ico natively, so we use a 32x32 PNG which all modern browsers accept
    await sharp(source)
        .resize(32, 32, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
        .png({ compressionLevel: 9 })
        .toFile(path.join(outputDir, 'favicon.ico'));
    console.log('  -> favicon.ico (32x32 PNG)');

    console.log('\nDone. Favicons generated.');
}

generateFavicons().catch(err => {
    console.error('Fatal error generating favicons:', err);
    process.exit(1);
});
