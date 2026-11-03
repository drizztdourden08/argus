/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import { recordMenuGroups } from '../../../../../shell';
import { copyText } from '../../../shared';
import { PLAN_KIND } from '../model';

const planMenuGroups = (code: string, open: () => void): MenuGroup[] => recordMenuGroups(PLAN_KIND, code, { open, copy: () => copyText(code) });

export { planMenuGroups };
