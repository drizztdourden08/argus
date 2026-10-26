/* @layer renderer-app @kind logic */
const criterionKeys = (texts: readonly string[]): string[] => {
  const seen = new Map<string, number>();
  return texts.map((text) => {
    const count = seen.get(text) ?? 0;
    seen.set(text, count + 1);
    return `${text}#${count}`;
  });
};

export { criterionKeys };
