/* @layer tooling-scripts @kind logic */
const dependentsOf = (crates, name, depth) => {
  const reached = new Set();
  let frontier = [name];
  for (let level = 0; level < depth && frontier.length > 0; level += 1) {
    const next = crates
      .filter((crate) => !reached.has(crate.name) && crate.name !== name)
      .filter((crate) => crate.deps.some((dep) => frontier.includes(dep)))
      .map((crate) => crate.name);
    next.forEach((found) => reached.add(found));
    frontier = next;
  }
  return [...reached];
};

export { dependentsOf };
