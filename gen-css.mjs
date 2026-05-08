import { compile } from "@tailwindcss/node";
import { Scanner } from "@tailwindcss/oxide";
import { readFileSync, writeFileSync } from "fs";
import { resolve, dirname } from "path";

const root = "/home/fufengyuan/WebstormProjects/tauri-vue-app-main";

// Read all Vue files and extract class names
const { execSync } = await import("child_process");
const files = execSync(
  `find ${root}/src -name "*.vue" -o -name "*.ts" -o -name "*.js" | head -500`,
  { encoding: "utf-8" }
).trim().split("\n").filter(Boolean);

// Extract class names from template and script
const classRegex = /class=["']([^"']+)["']/g;
const classBindRegex = /:class=["'\[][^"\]]+/g;
const allClasses = new Set();

for (const file of files) {
  try {
    const content = readFileSync(file, "utf-8");
    for (const match of content.matchAll(classRegex)) {
      match[1].split(/\s+/).forEach((c) => c && allClasses.add(c));
    }
  } catch {}
}

console.log(`Found ${allClasses.size} unique class names`);

// Generate CSS using Tailwind compile API
const candidates = [...allClasses].join(" ");
const result = await compile(`@import "tailwindcss";\n`, {
  base: root,
  onDependency: () => {},
});

const built = result.build(candidates.split(" "));
writeFileSync(resolve(root, "src/styles.generated.css"), built);
console.log(`Generated CSS: ${built.length} bytes`);