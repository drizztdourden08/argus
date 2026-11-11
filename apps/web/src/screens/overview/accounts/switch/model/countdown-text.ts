/* @layer renderer-app @kind logic */
const countdownText = (leftMs: number): string => {
  const seconds = Math.max(0, Math.ceil(leftMs / 1000));
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
};

export { countdownText };
