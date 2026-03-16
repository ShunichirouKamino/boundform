const https = require("https");
const http = require("http");
const fs = require("fs");
const path = require("path");
const os = require("os");
const { execSync } = require("child_process");

const REPO = "ShunichirouKamino/boundform";
const BINARY_NAME = "boundform";

// Map Node.js platform/arch to Rust target and binary name
function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();

  const targets = {
    "win32-x64": {
      target: "x86_64-pc-windows-gnu",
      binary: "boundform.exe",
      asset: "boundform-x86_64-pc-windows-gnu.exe",
    },
    "linux-x64": {
      target: "x86_64-unknown-linux-gnu",
      binary: "boundform",
      asset: "boundform-x86_64-unknown-linux-gnu",
    },
    "darwin-x64": {
      target: "x86_64-apple-darwin",
      binary: "boundform",
      asset: "boundform-x86_64-apple-darwin",
    },
    "darwin-arm64": {
      target: "aarch64-apple-darwin",
      binary: "boundform",
      asset: "boundform-aarch64-apple-darwin",
    },
  };

  const key = `${platform}-${arch}`;
  const info = targets[key];

  if (!info) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
        `Supported: ${Object.keys(targets).join(", ")}`
    );
  }

  return info;
}

function getCacheDir() {
  const dir = path.join(os.homedir(), ".cache", "boundform");
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

function getVersion() {
  const pkg = require("../package.json");
  return pkg.version;
}

function getBinaryPath() {
  const info = getPlatformInfo();
  const version = getVersion();
  return path.join(getCacheDir(), version, info.binary);
}

function followRedirects(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith("https") ? https : http;
    client
      .get(url, { headers: { "User-Agent": "boundform-npm" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          followRedirects(res.headers.location).then(resolve).catch(reject);
        } else if (res.statusCode === 200) {
          resolve(res);
        } else {
          reject(new Error(`HTTP ${res.statusCode} fetching ${url}`));
        }
      })
      .on("error", reject);
  });
}

async function downloadBinary() {
  const info = getPlatformInfo();
  const version = getVersion();
  const tag = `v${version}`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${info.asset}`;

  const binaryPath = getBinaryPath();
  const binaryDir = path.dirname(binaryPath);
  fs.mkdirSync(binaryDir, { recursive: true });

  console.log(`Downloading boundform ${tag} for ${os.platform()}-${os.arch()}...`);

  const res = await followRedirects(url);

  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(binaryPath);
    res.pipe(file);
    file.on("finish", () => {
      file.close();
      // Make executable on Unix
      if (os.platform() !== "win32") {
        fs.chmodSync(binaryPath, 0o755);
      }
      console.log(`Downloaded to ${binaryPath}`);
      resolve(binaryPath);
    });
    file.on("error", (err) => {
      fs.unlinkSync(binaryPath);
      reject(err);
    });
  });
}

async function ensureBinary() {
  const binaryPath = getBinaryPath();
  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }
  return downloadBinary();
}

module.exports = { getBinaryPath, ensureBinary, getPlatformInfo };
