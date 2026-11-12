import fs from "node:fs";
import { pathToFileURL } from "node:url";

const modulePath = process.env.RESVG_MODULE;
if (!modulePath) throw new Error("RESVG_MODULE is required");
const { Resvg } = await import(pathToFileURL(modulePath));
const file = process.argv[2];
const catalogue = JSON.parse(fs.readFileSync(file, "utf8"));
for (const icon of catalogue.icons) {
  const svg = icon.svg.replace("<svg ", '<svg width="72" height="72" ');
  const png = new Resvg(svg, { fitTo: { mode: "width", value: 72 } }).render().asPng();
  icon.preview = Buffer.from(png).toString("base64");
}
fs.writeFileSync(file, JSON.stringify(catalogue) + "\n");
console.log(`Rendered ${catalogue.icons.length} icon previews`);
