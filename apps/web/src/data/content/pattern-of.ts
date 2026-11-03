/* @layer renderer-app @kind logic */
const patternOf = (source: string): RegExp => new RegExp(source, 'u');

export { patternOf };
