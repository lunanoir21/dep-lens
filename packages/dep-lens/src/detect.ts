import { existsSync } from 'node:fs';
import path from 'node:path';

export interface EcosystemSignature {
  id: string;
  label: string;
  files: string[];
}

/** Manifest files dep-lens looks for, one entry per supported ecosystem. */
export const ECOSYSTEM_SIGNATURES: readonly EcosystemSignature[] = [
  { id: 'npm', label: 'npm / yarn / pnpm', files: ['package.json'] },
  { id: 'cargo', label: 'Cargo', files: ['Cargo.toml'] },
  { id: 'go', label: 'Go', files: ['go.mod'] },
  {
    id: 'python',
    label: 'Python',
    files: ['pyproject.toml', 'requirements.txt', 'Pipfile', 'poetry.lock', 'uv.lock'],
  },
  { id: 'ruby', label: 'Ruby', files: ['Gemfile.lock'] },
  { id: 'php', label: 'PHP', files: ['composer.lock'] },
  {
    id: 'java',
    label: 'Java',
    files: ['pom.xml', 'build.gradle', 'build.gradle.kts', 'gradle.lockfile'],
  },
  { id: 'dart', label: 'Dart / Flutter', files: ['pubspec.lock'] },
  { id: 'cpp', label: 'C/C++', files: ['vcpkg.json', 'conanfile.txt'] },
];

/** Ecosystem signatures whose manifest files are present under `dir`. */
export function detectEcosystems(dir: string): EcosystemSignature[] {
  return ECOSYSTEM_SIGNATURES.filter((eco) =>
    eco.files.some((file) => existsSync(path.join(dir, file))),
  );
}
