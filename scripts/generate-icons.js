import fs from 'fs';
import pngToIco from 'png-to-ico';

console.log("Generating icon.ico...");

pngToIco('src/assets/logo.png')
    .then(buf => {
        fs.writeFileSync('src-tauri/icons/icon.ico', buf);
        console.log("Created src-tauri/icons/icon.ico");
    })
    .catch(console.error);

// Also copy for other formats (simple fs copy)
// We need to use fs.copyFileSync
fs.copyFileSync('src/assets/logo.png', 'src-tauri/icons/icon.png');
fs.copyFileSync('src/assets/logo.png', 'src-tauri/icons/32x32.png');
fs.copyFileSync('src/assets/logo.png', 'src-tauri/icons/128x128.png');
fs.copyFileSync('src/assets/logo.png', 'src-tauri/icons/Square150x150Logo.png');
fs.copyFileSync('src/assets/logo.png', 'src-tauri/icons/Square44x44Logo.png');
console.log("Copied PNG icons.");
