/* @layer renderer-app @kind logic */
const keyStep = (key: string): 1 | -1 | null => {
  if (key === 'ArrowLeft') return -1;
  if (key === 'ArrowRight') return 1;
  return null;
};

export { keyStep };
