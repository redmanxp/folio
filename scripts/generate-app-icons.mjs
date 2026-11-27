import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Resvg } from "@resvg/resvg-js";
import sharp from "sharp";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const svgPath = join(root, "src", "assets", "brand", "folio_icon.svg");
const outDir = join(root, "brand");
const pngMaster = join(outDir, "folio_icon_1024.png");

mkdirSync(outDir, { recursive: true });

let svg = readFileSync(svgPath, "utf8");
if (svg.charCodeAt(0) === 0xfeff) svg = svg.slice(1);

const resvg = new Resvg(svg, {
  fitTo: { mode: "width", value: 880 },
  background: "rgba(0,0,0,0)",
});
const rendered = resvg.render();
const iconPng = rendered.asPng();

await sharp({
  create: {
    width: 1024,
    height: 1024,
    channels: 4,
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  },
})
  .composite([{ input: iconPng, gravity: "centre" }])
  .png()
  .toFile(pngMaster);

await sharp(pngMaster).resize(32, 32).png().toFile(join(root, "public", "folio_icon_32.png"));
await sharp(pngMaster).resize(192, 192).png().toFile(join(root, "public", "folio_icon_192.png"));

writeFileSync(
  join(outDir, "README.md"),
  `# Brand Folio

- \`folio_icon.svg\` — símbolo (favicon, app, menú)
- \`folio_wordmark.svg\` — solo nombre
- \`folio_full.svg\` — logo principal
- \`folio_icon_1024.png\` — master para \`npm run icons:tauri\`

Paleta: #2563EB · #4F8DF7 · #20B486 · #172B4D · #CBD5E1
`,
);

console.log("Generado:", pngMaster);
