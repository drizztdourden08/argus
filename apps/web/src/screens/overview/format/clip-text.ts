/* @layer renderer-app @kind logic */
const clipText = (text: string, limit: number): string =>
  text.length > limit ? `${text.slice(0, limit - 1)}...` : text;

export { clipText };
