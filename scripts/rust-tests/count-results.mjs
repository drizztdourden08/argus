/* @layer tooling-scripts @kind logic */
const resultLine = /^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored/;

const countResults = (text, counts) => {
  const lines = text.split(/\r?\n/);
  const carry = lines.pop() ?? '';
  for (const line of lines) {
    const match = resultLine.exec(line);
    if (!match) continue;
    counts.passed += Number(match[1]);
    counts.failed += Number(match[2]);
    counts.ignored += Number(match[3]);
  }
  return carry;
};

export { countResults };
