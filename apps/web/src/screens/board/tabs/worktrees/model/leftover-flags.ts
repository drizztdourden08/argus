/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import type { LeftoverFlag } from './model.type';

const leftoverFlags = (row: Worktree): LeftoverFlag[] => {
  const facts = row.leftover;
  if (!facts) return [];
  const flags: LeftoverFlag[] = [];
  if (facts.hasNestedGit) {
    flags.push({ text: 'nested git', tip: `Contains a nested git repository (${facts.nestedGit ?? '.git'}), which may hold real work: Argus does not delete it.` });
  }
  if (facts.isLink) flags.push({ text: 'link', tip: 'A link or junction: Argus never follows or deletes one.' });
  if (facts.nodeModules) {
    flags.push({ text: 'node_modules', tip: 'It holds a node_modules folder: its size is not counted (only its top-level entries).' });
  }
  if (facts.pending) flags.push({ text: 'measuring', tip: 'Not measured yet: the next scan measures it.' });
  else if (facts.partial) flags.push({ text: 'partial', tip: 'Too big to look through completely: the size is a lower bound.' });
  return flags;
};

export { leftoverFlags };
