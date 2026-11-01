/* @layer tooling-scripts @kind logic */
const visibility = '(?:pub(?:\\([^)]*\\))?\\s+)?';
const pathAttribute = /^\s*#\[\s*path\s*=\s*"([^"]+)"\s*\]\s*$/;
const anyAttribute = /^\s*#\[/;
const sameLine = new RegExp(`#\\[\\s*path\\s*=\\s*"([^"]+)"\\s*\\]\\s*${visibility}mod\\s+(\\w+)\\s*;`);
const plain = new RegExp(`^\\s*${visibility}mod\\s+(\\w+)\\s*;`);

const nextPending = (line, pending) => {
  const attr = pathAttribute.exec(line);
  if (attr) return attr[1];
  if (anyAttribute.test(line) || line.trim() === '') return pending;
  return null;
};

const readDeclarations = (text) => {
  const found = [];
  let pending = null;
  for (const line of text.split(/\r?\n/)) {
    const inline = sameLine.exec(line);
    const declared = plain.exec(line);
    if (inline) found.push({ name: inline[2], path: inline[1] });
    else if (declared) found.push({ name: declared[1], path: pending });
    pending = inline ? null : nextPending(line, pending);
  }
  return found;
};

export { readDeclarations };
