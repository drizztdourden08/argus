/* @layer renderer-app @kind logic */
const linkOf = (id: string): string => `${window.location.href.split('#')[0] ?? ''}#${id}`;

export { linkOf };
