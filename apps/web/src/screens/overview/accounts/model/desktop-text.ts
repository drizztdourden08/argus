/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';

const desktopText = (login: AccountDoc['desktopLogin']): string => (login?.saved ? 'Saved' : 'Not saved');

export { desktopText };
