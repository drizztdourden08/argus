/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useSearchParams } from 'react-router';
import { SWITCH_LINK_PARAM } from '../switch.constants';
import type { SwitchLink } from '../switch.type';

const useSwitchLink = (): SwitchLink => {
  const [params, setParams] = useSearchParams();
  const target = params.get(SWITCH_LINK_PARAM);
  const clear = useCallback(() => {
    setParams((was) => {
      const next = new URLSearchParams(was);
      next.delete(SWITCH_LINK_PARAM);
      return next;
    }, { replace: true });
  }, [setParams]);
  return { target, clear };
};

export { useSwitchLink };
