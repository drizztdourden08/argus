/* @layer renderer-app @kind logic */
const folderOf = (path: string): string => path.replace(/[\\/]+$/, '').split(/[\\/]/).at(-1) ?? path;

export { folderOf };
