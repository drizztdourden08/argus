/* @layer renderer-app @kind logic */
const scopePath = (scope: string, ...parts: string[]): string =>
  ['/api/p', scope, ...parts].map((part, index) => (index === 0 ? part : encodeURIComponent(part))).join('/');

export { scopePath };
