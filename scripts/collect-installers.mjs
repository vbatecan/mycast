import { copyFile, mkdir, readdir } from 'node:fs/promises';
import { join } from 'node:path';

const windows = process.platform === 'win32';
const formats = windows
  ? [['nsis', '.exe']]
  : [
      ['deb', '.deb'],
      ['appimage', '.AppImage'],
    ];
await mkdir('release-assets', { recursive: true });
for (const [directory, extension] of formats) {
  const source = join('src-tauri', 'target', 'release', 'bundle', directory);
  const files = (await readdir(source)).filter((file) =>
    file.endsWith(extension),
  );
  if (files.length !== 1)
    throw new Error(
      `Expected one ${extension} installer in ${source}, found ${files.length}`,
    );
  await copyFile(join(source, files[0]), join('release-assets', files[0]));
  console.log(`Collected ${files[0]}`);
}
