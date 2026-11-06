import { readdir, readFile, writeFile } from 'node:fs/promises';
const version = JSON.parse(await readFile('package.json', 'utf8')).version;
const tag = process.env.RELEASE_TAG;
if (tag !== `v${version}`) throw new Error('Updater tag/version mismatch');
const files = await readdir('release-assets');
const platforms = {};
for (const [platform, extension] of [
  ['windows-x86_64', '-setup.exe'],
  ['linux-x86_64', '.AppImage'],
]) {
  const matches = files.filter((name) => name.endsWith(extension));
  if (matches.length !== 1)
    throw new Error(`Expected exactly one updater for ${platform}`);
  const name = matches[0];
  const signature = (
    await readFile(`release-assets/${name}.sig`, 'utf8')
  ).trim();
  if (!signature) throw new Error(`Missing updater signature for ${platform}`);
  platforms[platform] = {
    signature,
    url: `https://github.com/${process.env.GITHUB_REPOSITORY}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(name)}`,
  };
}
await writeFile(
  'release-assets/latest.json',
  JSON.stringify(
    {
      version,
      notes: await readFile('docs/release-notes.md', 'utf8'),
      pub_date: new Date().toISOString(),
      platforms,
    },
    null,
    2,
  ),
);
