/* @layer renderer-app @kind logic */
const folderOf = (path: string): string => path.split(/[\\/]/u).filter(Boolean).pop() ?? path;

export { folderOf };
