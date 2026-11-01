/* @layer tooling-scripts @kind logic */
const ignoreProblem = (info) => {
  if (info.tests === 0 || info.tags[0] !== 'e2e') return null;
  if (info.ignores >= info.tests) return null;
  return `e2e file with ${info.tests} tests but ${info.ignores} #[ignore] attributes`;
};

export { ignoreProblem };
