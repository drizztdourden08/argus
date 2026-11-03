/* @layer tooling-scripts @kind logic */
const testAttribute = /^\s*#\[(?:tokio::)?test\b/;
const ignoreAttribute = /^\s*#\[ignore\b/;
const tagLine = /^\s*\/\/\s*test-tag:\s*(.*?)\s*$/;

const readTags = (text) => {
  const info = { tests: 0, ignores: 0, tags: [] };
  for (const line of text.split(/\r?\n/)) {
    const tag = tagLine.exec(line);
    if (tag) info.tags.push(tag[1]);
    if (testAttribute.test(line)) info.tests += 1;
    if (ignoreAttribute.test(line)) info.ignores += 1;
  }
  return info;
};

export { readTags };
