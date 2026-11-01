/* @layer tooling-scripts @kind logic */
const keyOf = (occurrence) => `${occurrence.crate}|${occurrence.target.kind}|${occurrence.target.name}|${occurrence.file}|${occurrence.modPath}`;

export { keyOf };
