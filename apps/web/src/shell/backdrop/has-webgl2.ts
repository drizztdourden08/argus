/* @layer renderer-app @kind logic */
const hasWebgl2 = (): boolean => {
  try {
    return Boolean(document.createElement('canvas').getContext('webgl2'));
  } catch {
    return false;
  }
};

export { hasWebgl2 };
