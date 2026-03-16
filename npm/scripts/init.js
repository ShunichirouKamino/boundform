const fs = require("fs");
const path = require("path");

const cwd = process.cwd();

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

  // 1. Copy skills to .claude/skills/
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

  // 2. Create boundform.yml template
  const templateSrc = path.join(__dirname, "..", "templates", "boundform.yml");
  const templateDest = path.join(cwd, "boundform.yml");

  if (fs.existsSync(templateDest)) {
    console.log("  [skip] boundform.yml (already exists)");
  } else if (fs.existsSync(templateSrc)) {
    fs.copyFileSync(templateSrc, templateDest);
    console.log("  [created] boundform.yml");
  }

  // 3. Update .gitignore
  const gitignorePath = path.join(cwd, ".gitignore");
  const ignoreEntry = ".claude/skills/*-workspace/";

  if (fs.existsSync(gitignorePath)) {
    const content = fs.readFileSync(gitignorePath, "utf-8");
    if (!content.includes(ignoreEntry)) {
      fs.appendFileSync(gitignorePath, `\n# boundform skill workspaces\n${ignoreEntry}\n`);
      console.log("  [updated] .gitignore");
    } else {
      console.log("  [skip] .gitignore (already configured)");
    }
  }

  console.log("\nDone! Next steps:");
  console.log("  1. Edit boundform.yml with your form constraints");
  console.log("  2. Run: npx boundform --config boundform.yml");
  console.log("  3. Use /boundform-guide in Claude Code for help");
}

init();
