const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const inputDir = path.join(__dirname, '../static/pictures');
const outputDir = path.join(__dirname, '../static/pictures/optimized');

if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

// Collect image files from a directory, returning { file, fullPath, prefix }
function collectImages(dir, prefix) {
    if (!fs.existsSync(dir)) return [];
    return fs.readdirSync(dir).filter(file => {
        const ext = path.extname(file).toLowerCase();
        if (!['.jpg', '.jpeg', '.png'].includes(ext)) return false;
        if (file.endsWith('~')) return false;
        if (file.includes('.orig.')) return false;
        return true;
    }).map(file => ({ file, fullPath: path.join(dir, file), prefix }));
}

// Find all image files from top-level directory
// To add subdirectories, append entries like:
//   ...collectImages(path.join(inputDir, 'mysubdir'), 'mysubdir'),
const allFiles = [
    ...collectImages(inputDir, ''),
];

// Build a map: basename -> source file path
// If a _sharpen variant exists, prefer it as the high-res source
const sourceMap = new Map();

for (const { file, fullPath, prefix } of allFiles) {
    const ext = path.extname(file);
    const nameWithoutExt = path.basename(file, ext);
    const sharpenMatch = nameWithoutExt.match(/^(.+)_sharpen$/);
    const rawBase = sharpenMatch ? sharpenMatch[1] : nameWithoutExt;
    const basename = prefix ? `${prefix}_${rawBase}` : rawBase;

    if (sharpenMatch) {
        sourceMap.set(basename, { file, path: fullPath });
    } else if (!sourceMap.has(basename)) {
        sourceMap.set(basename, { file, path: fullPath });
    }
}

const sizes = [400, 800, 1200];

async function processImages() {
    for (const [basename, source] of sourceMap) {
        console.log(`Processing: ${source.file} -> ${basename}`);

        for (const size of sizes) {
            await sharp(source.path)
                .resize(size)
                .webp({ quality: 80 })
                .toFile(path.join(outputDir, `${basename}_${size}w.webp`));

            await sharp(source.path)
                .resize(size)
                .jpeg({ quality: 80 })
                .toFile(path.join(outputDir, `${basename}_${size}w.jpg`));
        }
        console.log(`  -> Generated 6 variants for ${basename}`);
    }
    console.log(`\nDone. Processed ${sourceMap.size} images.`);
}

processImages().catch(err => {
    console.error('Fatal error in image optimization:', err);
    process.exit(1);
});
