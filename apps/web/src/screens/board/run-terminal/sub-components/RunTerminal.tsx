/* @layer renderer-app @kind component */
import { Button, Card, Flex, Icon, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { RUN_LOG_TAIL } from '../../../../data/runs';
import { TERMINAL_HEIGHT } from '../run-terminal.constants';
import type { RunTerminalProps } from '../run-terminal.type';
import { TerminalBody } from './TerminalBody';
import { TerminalSummary } from './TerminalSummary';

const RunTerminal = (props: RunTerminalProps) => {
  const { run, boardKey, onLog } = props;
  const [open, setOpen] = useState(false);
  return (
    <Card>
      <Flex direction="column" gap="xs">
        <Flex gap="sm" align="center" justify="between" wrap>
          <Flex gap="sm" align="center" wrap>
            <Button size="xs" variant="ghost" aria-expanded={open} icon={<Icon name={open ? 'chevron-down' : 'chevron-right'} />} onClick={() => setOpen(!open)}>
              <Text.B>Terminal</Text.B>
            </Button>
            <TerminalSummary run={run} />
          </Flex>
          <Button size="xs" variant="ghost" title={`Open the full log of ${run.id}`} aria-label={`Full log of ${run.id}`} onClick={() => onLog(run.id)}>Full log</Button>
        </Flex>
        {open && <TerminalBody run={run} boardKey={boardKey} limit={RUN_LOG_TAIL} height={TERMINAL_HEIGHT} />}
      </Flex>
    </Card>
  );
};

export { RunTerminal };
