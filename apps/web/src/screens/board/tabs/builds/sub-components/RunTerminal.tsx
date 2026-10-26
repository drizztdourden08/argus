/* @layer renderer-app @kind component */
import { Disclosure } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { RUN_LOG_TAIL } from '../../../../../data/runs';
import { TERMINAL_HEIGHT } from '../builds.constants';
import type { RunTerminalProps } from '../builds-props.type';
import { TerminalBody } from './TerminalBody';
import { TerminalSummary } from './TerminalSummary';

const RunTerminal = (props: RunTerminalProps) => {
  const { run, boardKey } = props;
  const [open, setOpen] = useState(false);
  return (
    <Disclosure size="sm" open={open} onOpenChange={setOpen} summary={<TerminalSummary run={run} />}>
      {open && <TerminalBody run={run} boardKey={boardKey} limit={RUN_LOG_TAIL} height={TERMINAL_HEIGHT} />}
    </Disclosure>
  );
};

export { RunTerminal };
