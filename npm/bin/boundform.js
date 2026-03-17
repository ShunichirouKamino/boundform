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

    // Filter environment variables to avoid leaking secrets to the binary.
    // Only pass PATH and common locale/terminal variables.
    const ALLOWED_ENV_PREFIXES = ["PATH", "HOME", "USER", "LANG", "LC_", "TERM", "COLORTERM", "NO_COLOR", "FORCE_COLOR"];
    const ALLOWED_ENV_EXACT = new Set(["HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy", "NO_PROXY", "no_proxy"]);
    const filteredEnv = {};
    for (const [key, value] of Object.entries(process.env)) {
      if (
        ALLOWED_ENV_EXACT.has(key) ||
        ALLOWED_ENV_PREFIXES.some((prefix) => key === prefix || key.startsWith(prefix))
      ) {
        filteredEnv[key] = value;
      }
    }
    // Windows needs SystemRoot and APPDATA
    if (process.platform === "win32") {
      for (const key of ["SystemRoot", "APPDATA", "LOCALAPPDATA", "TEMP", "TMP", "USERPROFILE"]) {
        if (process.env[key]) filteredEnv[key] = process.env[key];
      }
    }

    const child = spawn(binaryPath, args, {
      stdio: "inherit",
      env: filteredEnv,
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
