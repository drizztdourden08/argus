/* @layer renderer-app @kind logic */
const configParseList = (text: string): string[] =>
  text
    .split(',')
    .map((part) => part.trim())
    .filter((part) => part !== '');

export { configParseList };
