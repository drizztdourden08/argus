/* @layer tooling-scripts @kind logic */
const usesModule = (occurrence, modPath) => {
  const top = modPath.split('::')[0];
  if (top === '' || occurrence.modPath.startsWith(`${modPath}::`)) return true;
  return new RegExp(`\\b${top}\\b`).test(occurrence.text);
};

export { usesModule };
