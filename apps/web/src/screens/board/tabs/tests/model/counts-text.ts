/* @layer renderer-app @kind logic */
import type { Counts } from './model.type';

const PARTS: [keyof Counts, string][] = [
  ['run', 'to run'],
  ['pass', 'pass'],
  ['fail', 'fail'],
  ['partial', 'partial'],
  ['blocked', 'blocked'],
];

const countsText = (counts: Counts): string => {
  const parts = PARTS.filter(([key]) => counts[key] > 0).map(([key, word]) => `${counts[key]} ${word}`);
  return parts.length > 0 ? parts.join(' · ') : 'no tests yet';
};

export { countsText };
