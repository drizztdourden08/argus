/* @layer tooling-scripts @kind logic */
import { tagProblem } from './tag-problem.mjs';

const tagProblems = (selected) => {
  const seen = new Map();
  for (const occurrence of selected) {
    const problem = tagProblem(occurrence.info);
    if (problem) seen.set(occurrence.file, problem);
  }
  return [...seen].map(([file, problem]) => `${file}: ${problem}`);
};

export { tagProblems };
