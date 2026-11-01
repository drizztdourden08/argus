/* @layer tooling-scripts @kind logic */
const validTags = ['critical', 'standard', 'e2e'];

const tagProblem = (info) => {
  if (info.tests === 0) return null;
  if (info.tags.length === 0) return 'no test-tag line';
  if (info.tags.length > 1) return `${info.tags.length} test-tag lines (${info.tags.join(', ')})`;
  if (!validTags.includes(info.tags[0])) return `unknown tag "${info.tags[0]}" (use critical, standard or e2e)`;
  return null;
};

export { tagProblem };
