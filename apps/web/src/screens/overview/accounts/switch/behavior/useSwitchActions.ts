/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { usePostAction } from '../../behavior/usePostAction';
import { ANYWAY_PATH, ANYWAY_TOAST, CANCEL_PATH, SWITCH_CANCELLED_TOAST, SWITCH_PATH, SWITCH_STARTED } from '../switch.constants';
import type { SwitchActions } from '../switch.type';

const useSwitchActions = (): SwitchActions => {
  const { busy, post } = usePostAction();
  const start = useCallback((to: string, handover: boolean) => post(SWITCH_PATH, { to, handover }, SWITCH_STARTED), [post]);
  const anyway = useCallback(() => post(ANYWAY_PATH, undefined, ANYWAY_TOAST), [post]);
  const cancel = useCallback(() => post(CANCEL_PATH, undefined, SWITCH_CANCELLED_TOAST), [post]);
  return { busy, start, anyway, cancel };
};

export { useSwitchActions };
