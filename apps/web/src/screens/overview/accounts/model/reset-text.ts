/* @layer renderer-app @kind logic */
import { RESET_FORMAT } from '../accounts.constants';

const resetText = (resetsAt: string | null | undefined): string => {
  const at = resetsAt ? new Date(resetsAt) : null;
  if (!at || Number.isNaN(at.getTime())) return '';
  return `resets ${at.toLocaleString('en-CA', RESET_FORMAT)}`;
};

export { resetText };
