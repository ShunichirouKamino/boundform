const fs = require("fs");
const path = require("path");

const cwd = process.cwd();
const BOUNDFORM_DIR = path.join(cwd, "boundform");

function copyDir(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

function init() {
  console.log("Initializing boundform...\n");

  // 1. Create boundform/ directory
  fs.mkdirSync(BOUNDFORM_DIR, { recursive: true });

  // 2. Create boundform.yml template in boundform/
  const templateSrc = path.join(__dirname, "..", "templates", "boundform.yml");
  const templateDest = path.join(BOUNDFORM_DIR, "boundform.yml");

  if (fs.existsSync(templateDest)) {
    console.log("  [skip] boundform/boundform.yml (already exists)");
  } else if (fs.existsSync(templateSrc)) {
    fs.copyFileSync(templateSrc, templateDest);
    console.log("  [created] boundform/boundform.yml");
  }

  // 3. Copy skills to .claude/skills/
  const skillsSrc = path.join(__dirname, "..", "skills");
  const skillsDest = path.join(cwd, ".claude", "skills");

  if (fs.existsSync(skillsSrc)) {
    const skills = fs.readdirSync(skillsSrc, { withFileTypes: true })
      .filter((d) => d.isDirectory());

    for (const skill of skills) {
      const dest = path.join(skillsDest, skill.name);
      if (fs.existsSync(dest)) {
        console.log(`  [skip] .claude/skills/${skill.name}/ (already exists)`);
      } else {
        copyDir(path.join(skillsSrc, skill.name), dest);
        console.log(`  [created] .claude/skills/${skill.name}/`);
      }
    }
  }

  console.log("\nDone! Next steps:");
  console.log("  1. Edit boundform/boundform.yml with your form constraints");
  console.log("  2. Run: npx boundform --config boundform/boundform.yml");
  console.log("  3. Use /boundform-guide in Claude Code for help");
}

init();
