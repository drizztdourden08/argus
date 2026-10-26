/* @layer renderer-app @kind logic */
const clipText = (text: string, max: number): string => (text.length > max ? `${text.slice(0, max).trimEnd()}...` : text);

export { clipText };
