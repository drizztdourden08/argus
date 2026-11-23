/* @layer renderer-app @kind logic */
const configToggleList = (list: readonly string[], value: string, on: boolean): string[] =>
  on ? [...list.filter((entry) => entry !== value), value] : list.filter((entry) => entry !== value);

export { configToggleList };
