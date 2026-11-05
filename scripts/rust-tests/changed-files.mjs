/* @layer tooling-scripts @kind logic */
import { execFileSync } from 'node:child_process';

const git = (root, args) => execFileSync('git', args, { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true });

const lines = (text) => text.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);

const mergeBase = (root) => {
  for (const branch of ['main', 'origin/main']) {
    try {
      return git(root, ['merge-base', 'HEAD', branch]).trim();
    } catch {
      continue;
    }
  }
  throw new Error('no merge base against main or origin/main; pass --base <ref>');
};

const changedFiles = (root, base) => {
  const from = base ?? mergeBase(root);
  const tracked = lines(git(root, ['diff', '--name-only', from]));
  const untracked = lines(git(root, ['ls-files', '--others', '--exclude-standard']));
  return [...new Set([...tracked, ...untracked])];
};

export { changedFiles };
