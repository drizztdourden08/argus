/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import { readUsage } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import type { ChatUsage } from '../OverlordChat.type';

const useChatUsage = (working: boolean): ChatUsage => {
  const { data, reload } = useRemoteRead(readUsage);
  const was = useRef(working);

  useEffect(() => {
    if (was.current && !working) reload();
    was.current = working;
  }, [working, reload]);

  return { usage: data, reload };
};

export { useChatUsage };
