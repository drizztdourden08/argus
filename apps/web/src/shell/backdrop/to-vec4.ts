/* @layer renderer-app @kind logic */
const toVec4 = (hex: string): [number, number, number, number] => {
  const value = Number.parseInt(hex.replace('#', ''), 16);
  return [((value >> 16) & 255) / 255, ((value >> 8) & 255) / 255, (value & 255) / 255, 1];
};

export { toVec4 };
