/* @layer renderer-app @kind logic */
import { STATUS_LOOKS } from './model.constants';
import type { StatusLook } from './model.type';

const statusLook = (status: string): StatusLook => STATUS_LOOKS[status] ?? { label: status, tone: 'neutral' };

export { statusLook };
