/* @layer tooling-scripts @kind logic */
import { listDir } from './list-dir.mjs';
import { ignoreProblem } from './ignore-problem.mjs';
import { readSource } from './source.mjs';
import { tagProblem } from './tag-problem.mjs';
import { readTags } from './tags.mjs';

const skipped = new Set(['target', 'node_modules', '.git']);

const rustFiles = (root, dir) => listDir(root, dir).flatMap((entry) => {
  if (skipped.has(entry.name)) return [];
  if (entry.isDirectory()) return rustFiles(root, `${dir}/${entry.name}`);
  return entry.name.endsWith('.rs') ? [`${dir}/${entry.name}`] : [];
});

const checkTags = (root) => {
  const problems = [];
  let withTests = 0;
  for (const file of rustFiles(root, 'crates')) {
    const info = readTags(readSource(root, file) ?? '');
    if (info.tests > 0) withTests += 1;
    const problem = tagProblem(info) ?? ignoreProblem(info);
    if (problem) problems.push(`${file}: ${problem}`);
  }
  return { problems, withTests };
};

export { checkTags };
