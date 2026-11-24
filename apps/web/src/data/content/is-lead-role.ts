/* @layer renderer-app @kind logic */
import { LEAD_ROLE } from './lead-role.constants';

const isLeadRole = (role: string | undefined | null): boolean => role === LEAD_ROLE;

export { isLeadRole };
