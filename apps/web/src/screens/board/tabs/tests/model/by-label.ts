/* @layer renderer-app @kind logic */
const byLabel = (by: string | null | undefined): string => {
  if (by === 'human') return 'you';
  return by ?? 'a thread';
};

export { byLabel };
