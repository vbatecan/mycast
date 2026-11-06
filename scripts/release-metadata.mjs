import { appendFile, readFile } from 'node:fs/promises';

const json = async (path) => JSON.parse(await readFile(path, 'utf8'));
const pkg = await json('package.json');
const lock = await json('package-lock.json');
const config = await json('src-tauri/tauri.conf.json');
const cargo = await readFile('src-tauri/Cargo.toml', 'utf8');
const cargoLock = await readFile('src-tauri/Cargo.lock', 'utf8');
const cargoVersion = cargo.match(
  /\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/,
)?.[1];
const lockedVersion = cargoLock.match(
  /\[\[package\]\]\s*name = "mycast"\s*version = "([^"]+)"/,
)?.[1];
const version = pkg.version;
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error(`Unsupported release version: ${version}`);
}
for (const [name, value] of Object.entries({
  'package-lock.json': lock.version,
  'package-lock.json root': lock.packages?.['']?.version,
  'tauri.conf.json': config.version,
  'Cargo.toml': cargoVersion,
  'Cargo.lock': lockedVersion,
})) {
  if (value !== version)
    throw new Error(
      `${name} version ${value} does not match package.json ${version}`,
    );
}
const ref = process.env.GITHUB_REF ?? '';
if (ref.startsWith('refs/tags/') && ref !== `refs/tags/v${version}`) {
  throw new Error(
    `Tag ${ref.slice(10)} must match v${version}. Update all package versions before tagging.`,
  );
}
if (process.env.GITHUB_OUTPUT) {
  await appendFile(
    process.env.GITHUB_OUTPUT,
    `version=${version}\nprerelease=${version.includes('-')}\n`,
  );
}
console.log(`Release version: ${version}`);
