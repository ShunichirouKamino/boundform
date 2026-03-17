const https = require("https");
const http = require("http");
const { URL } = require("url");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");
const os = require("os");

const REPO = "ShunichirouKamino/boundform";
const MAX_REDIRECTS = 5;
const CONNECT_TIMEOUT_MS = 30000;

/**
 * Resolve proxy URL from environment variables.
 * Checks HTTPS_PROXY, https_proxy, HTTP_PROXY, http_proxy (in that order).
 * Returns null if no proxy is configured or NO_PROXY matches the host.
 */
function getProxyUrl(targetUrl) {
  const noProxy = process.env.NO_PROXY || process.env.no_proxy || "";
  if (noProxy) {
    const { hostname } = new URL(targetUrl);
    const noProxyList = noProxy.split(",").map((s) => s.trim().toLowerCase());
    if (
      noProxyList.includes("*") ||
      noProxyList.some(
        (entry) => hostname === entry || hostname.endsWith(`.${entry}`)
      )
    ) {
      return null;
    }
  }

  const proxy =
    process.env.HTTPS_PROXY ||
    process.env.https_proxy ||
    process.env.HTTP_PROXY ||
    process.env.http_proxy ||
    null;
  return proxy || null;
}

/**
 * Make an HTTPS GET request, routing through a proxy if configured.
 * Uses HTTP CONNECT tunneling for HTTPS-over-proxy.
 * Returns a Promise that resolves with the response.
 */
function httpsGet(url, options = {}) {
  const proxyUrl = getProxyUrl(url);

  if (!proxyUrl) {
    return new Promise((resolve, reject) => {
      https
        .get(url, options, (res) => resolve(res))
        .on("error", reject);
    });
  }

  const proxy = new URL(proxyUrl);
  const target = new URL(url);

  return new Promise((resolve, reject) => {
    const connectHeaders = {
      Host: `${target.hostname}:${target.port || 443}`,
      "User-Agent": options.headers?.["User-Agent"] || "boundform-npm",
    };

    // Support proxy authentication via http://user:pass@proxy:port
    if (proxy.username) {
      const auth = Buffer.from(
        `${decodeURIComponent(proxy.username)}:${decodeURIComponent(proxy.password || "")}`
      ).toString("base64");
      connectHeaders["Proxy-Authorization"] = `Basic ${auth}`;
    }

    const connectReq = http.request({
      host: proxy.hostname,
      port: proxy.port || 80,
      method: "CONNECT",
      path: `${target.hostname}:${target.port || 443}`,
      headers: connectHeaders,
    });

    connectReq.setTimeout(CONNECT_TIMEOUT_MS, () => {
      connectReq.destroy(
        new Error(`Proxy CONNECT timed out after ${CONNECT_TIMEOUT_MS / 1000}s`)
      );
    });

    connectReq.on("connect", (connectRes, socket) => {
      if (connectRes.statusCode !== 200) {
        socket.destroy();
        reject(
          new Error(
            `Proxy CONNECT failed with status ${connectRes.statusCode}. ` +
              `Check proxy authentication and access to ${target.hostname}`
          )
        );
        return;
      }

      const tlsOptions = {
        ...options,
        socket,
        servername: target.hostname,
      };
      https
        .get(url, tlsOptions, (res) => resolve(res))
        .on("error", reject);
    });

    connectReq.on("error", reject);
    connectReq.end();
  });
}

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
  fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
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

/**
 * Follow HTTPS redirects with security constraints:
 * - Only follow HTTPS URLs (reject HTTP downgrade)
 * - Limit redirect depth to prevent infinite loops
 */
async function followRedirects(url, depth = 0) {
  if (depth > MAX_REDIRECTS) {
    throw new Error(`Too many redirects (max: ${MAX_REDIRECTS})`);
  }

  // Reject non-HTTPS URLs (prevent downgrade attacks)
  if (!url.startsWith("https://")) {
    throw new Error(
      `Refusing to follow non-HTTPS URL: ${url}. Only HTTPS is allowed for binary downloads.`
    );
  }

  const res = await httpsGet(url, {
    headers: { "User-Agent": "boundform-npm" },
  });

  if (
    res.statusCode >= 300 &&
    res.statusCode < 400 &&
    res.headers.location
  ) {
    return followRedirects(res.headers.location, depth + 1);
  } else if (res.statusCode === 200) {
    return res;
  } else {
    throw new Error(`HTTP ${res.statusCode} fetching ${url}`);
  }
}

/**
 * Fetch the SHA-256 checksum file from the GitHub Release.
 * The checksums file is expected at: boundform-checksums.sha256
 * Format: <hash>  <filename>
 */
async function fetchChecksum(tag, assetName) {
  const checksumUrl = `https://github.com/${REPO}/releases/download/${tag}/boundform-checksums.sha256`;

  try {
    const res = await followRedirects(checksumUrl);
    const data = await new Promise((resolve, reject) => {
      let body = "";
      res.on("data", (chunk) => (body += chunk));
      res.on("end", () => resolve(body));
      res.on("error", reject);
    });

    // Parse checksum file: each line is "<hash>  <filename>"
    for (const line of data.trim().split("\n")) {
      const parts = line.trim().split(/\s+/);
      if (parts.length >= 2 && parts[1] === assetName) {
        return parts[0].toLowerCase();
      }
    }

    console.warn(
      `Warning: checksum for ${assetName} not found in checksums file. Skipping verification.`
    );
    return null;
  } catch {
    console.warn(
      "Warning: checksums file not available. Skipping integrity verification."
    );
    return null;
  }
}

/**
 * Compute SHA-256 hash of a file.
 */
function computeFileHash(filePath) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash("sha256");
    const stream = fs.createReadStream(filePath);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", () => resolve(hash.digest("hex")));
    stream.on("error", reject);
  });
}

async function downloadBinary() {
  const info = getPlatformInfo();
  const version = getVersion();
  const tag = `v${version}`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${info.asset}`;

  const binaryPath = getBinaryPath();
  const binaryDir = path.dirname(binaryPath);
  fs.mkdirSync(binaryDir, { recursive: true, mode: 0o700 });

  console.log(
    `Downloading boundform ${tag} for ${os.platform()}-${os.arch()}...`
  );

  // Fetch expected checksum (if available)
  const expectedHash = await fetchChecksum(tag, info.asset);

  const res = await followRedirects(url);

  return new Promise((resolve, reject) => {
    const tmpPath = binaryPath + ".tmp";
    const file = fs.createWriteStream(tmpPath);
    res.pipe(file);
    file.on("finish", async () => {
      file.close();

      // Verify checksum if available
      if (expectedHash) {
        const actualHash = await computeFileHash(tmpPath);
        if (actualHash !== expectedHash) {
          fs.unlinkSync(tmpPath);
          reject(
            new Error(
              `Checksum mismatch!\n  Expected: ${expectedHash}\n  Actual:   ${actualHash}\n` +
                `The downloaded binary may have been tampered with. Aborting.`
            )
          );
          return;
        }
        console.log(`Checksum verified: ${actualHash.substring(0, 16)}...`);
      }

      // Move temp file to final path
      fs.renameSync(tmpPath, binaryPath);

      // Make executable on Unix
      if (os.platform() !== "win32") {
        fs.chmodSync(binaryPath, 0o755);
      }
      console.log(`Downloaded to ${binaryPath}`);
      resolve(binaryPath);
    });
    file.on("error", (err) => {
      try {
        fs.unlinkSync(tmpPath);
      } catch {}
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
