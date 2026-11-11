/* @layer renderer-app @kind logic */
import type { ReviewDiff } from '../../../../../data/types';

const nextChoice = (diff: ReviewDiff, current: string): string => {
  const at = diff.choices.findIndex((choice) => choice.id === current);
  return diff.choices[(at + 1) % diff.choices.length]?.id ?? diff.default;
};

export { nextChoice };
