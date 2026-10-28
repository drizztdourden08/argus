/* @layer renderer-app @kind util */
const rem = (count: number): string => `calc(var(--agents-rem) * ${count})`;

const columnWidth = (size: number, hideBelow: number, floor = 0): string =>
  `clamp(${rem(floor)}, calc((100cqi - ${rem(hideBelow)}) * 100), ${rem(size)})`;

export { columnWidth };
