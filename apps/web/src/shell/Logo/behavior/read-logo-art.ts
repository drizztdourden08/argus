/* @layer renderer-app @kind logic */
import { LOGO_CIRCLE_PATTERN, LOGO_PATH_PATTERN } from '../Logo.constants';
import type { LogoArt } from '../Logo.type';

const readLogoArt = (svg: string): LogoArt => ({
  d: [...svg.matchAll(LOGO_PATH_PATTERN)].map((match) => match[1] ?? ''),
  circles: [...svg.matchAll(LOGO_CIRCLE_PATTERN)].map((match) => ({ cx: Number(match[1]), cy: Number(match[2]), r: Number(match[3]) })),
});

export { readLogoArt };
