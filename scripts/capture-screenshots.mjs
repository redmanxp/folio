import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const demoHtml = path.join(__dirname, "demo-ui", "folio-demo.html");
const outDir = path.join(__dirname, "..", "docs", "assets", "screenshots");

const shots = [
  { view: "home", file: "inicio.png" },
  { view: "invoices", file: "facturas.png" },
  { view: "providers", file: "proveedores.png" },
];

await mkdir(outDir, { recursive: true });

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({
  viewport: { width: 1360, height: 860 },
  deviceScaleFactor: 2,
});

await page.goto(pathToFileURL(demoHtml).href, { waitUntil: "networkidle" });

for (const shot of shots) {
  await page.evaluate((view) => window.setDemoView(view), shot.view);
  await page.waitForTimeout(250);
  const target = page.locator("#app");
  await target.screenshot({
    path: path.join(outDir, shot.file),
    type: "png",
  });
  console.log(`OK ${shot.file}`);
}

await browser.close();
console.log(`Capturas en ${outDir}`);
