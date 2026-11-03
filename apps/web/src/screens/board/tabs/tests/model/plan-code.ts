/* @layer renderer-app @kind logic */
import { shownCode } from '../../../../../shell';
import { kindOf } from '../../../../../data/registry';
import { PLAN_KIND } from './model.constants';

const planCode = (id: string): string => {
  const kind = kindOf(PLAN_KIND);
  return kind ? shownCode(kind, id) : id;
};

export { planCode };
