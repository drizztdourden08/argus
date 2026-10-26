/* @layer renderer-app @kind logic */
const writeStored = (key: string, value: string): boolean => {
  try {
    localStorage.setItem(key, value);
    return true;
  } catch {
    return false;
  }
};

export { writeStored };
