/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import type { SwitchStage } from '../../../../../data/types';
import { usePostAction } from '../../behavior/usePostAction';
import {
  ANYWAY_PATH, ANYWAY_TOAST, CANCEL_PATH, SKIP_STEP_PATH, SKIP_STEP_TOAST, SKIP_THREAD_PATH, SKIP_THREAD_TOAST, SWITCH_CANCELLED_TOAST, SWITCH_PATH, SWITCH_STARTED,
} from '../switch.constants';
import type { SwitchActions } from '../switch.type';

const useSwitchActions = (): SwitchActions => {
  const { busy, post } = usePostAction();
  const start = useCallback((to: string, handover: boolean) => post(SWITCH_PATH, { to, handover }, SWITCH_STARTED), [post]);
  const anyway = useCallback(() => post(ANYWAY_PATH, undefined, ANYWAY_TOAST), [post]);
  const cancel = useCallback(() => post(CANCEL_PATH, undefined, SWITCH_CANCELLED_TOAST), [post]);
  const skipStep = useCallback((stage: SwitchStage) => post(SKIP_STEP_PATH, { stage }, SKIP_STEP_TOAST), [post]);
  const skipThread = useCallback((board: string, thread: string) => post(SKIP_THREAD_PATH, { board, thread }, SKIP_THREAD_TOAST), [post]);
  return { busy, start, anyway, cancel, skipStep, skipThread };
};

export { useSwitchActions };
