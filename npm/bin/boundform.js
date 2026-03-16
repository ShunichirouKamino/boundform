#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

// Handle `boundform init` separately
if (process.argv[2] === "init") {
  require("../scripts/init.js");
  process.exit(0);
}

const { getBinaryPath, ensureBinary } = require("../scripts/download-binary.js");

async function main() {
  try {
    const binaryPath = await ensureBinary();
    const args = process.argv.slice(2);

    const child = spawn(binaryPath, args, {
      stdio: "inherit",
      env: process.env,
    });

    child.on("error", (err) => {
      console.error(`Failed to execute boundform: ${err.message}`);
      process.exit(1);
    });

    child.on("exit", (code) => {
      process.exit(code ?? 0);
    });
  } catch (err) {
    console.error(err.message);
    process.exit(1);
  }
}

main();
