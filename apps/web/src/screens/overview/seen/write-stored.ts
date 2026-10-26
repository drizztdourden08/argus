/* @layer renderer-app @kind logic */
const writeStored = (key: string, millis: number): boolean => {
  try {
    localStorage.setItem(key, new Date(millis).toISOString());
    return true;
  } catch {
    return false;
  }
};

export { writeStored };
