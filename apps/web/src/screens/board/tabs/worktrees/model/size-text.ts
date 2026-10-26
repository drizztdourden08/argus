/* @layer renderer-app @kind logic */
import { GB_STEP, UNKNOWN_SIZE } from './model.constants';

const sizeText = (mb: number | null | undefined): string => {
  if (mb === null || mb === undefined) return UNKNOWN_SIZE;
  if (mb < 1) return 'under 1 MB';
  if (mb < 1024) return `${Math.round(mb)} MB`;
  return `${Math.round(mb / GB_STEP) / 10} GB`;
};

export { sizeText };
