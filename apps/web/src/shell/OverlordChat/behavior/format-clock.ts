/* @layer renderer-app @kind logic */
const MS_PER_SECOND = 1_000;
const SECONDS_PER_MINUTE = 60;

const formatClock = (millis: number): string => {
  const total = Math.round(millis / MS_PER_SECOND);
  const seconds = String(total % SECONDS_PER_MINUTE).padStart(2, '0');
  return `${Math.floor(total / SECONDS_PER_MINUTE)}:${seconds}`;
};

export { formatClock };
