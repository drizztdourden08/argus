/* @layer renderer-app @kind component */
import { Box, Flex } from '@drizztdourden08/tessera';
import { itemCode, RecordRow, useRecordPane } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import { RUN_KIND } from '../tests.constants';
import type { RunFrameProps } from '../tests-head-props.type';

const RunFrame = (props: RunFrameProps) => {
  const { view, open, onToggle, menu, signals, className, children } = props;
  const { boardKey, threads } = useTestsShared();
  const pane = useRecordPane();
  const { item } = view;
  const code = itemCode(item);
  return (
    <Box data-swap-item data-run={item.id} className={className}>
      <RecordRow
        head={{ kind: RUN_KIND, code, assignee: item.assignee ?? item.thread ?? '', boardKey, threads, menu, menuLabel: `More actions for ${code}` }}
        title={item.title || view.plan?.title || ''}
        time={item.updatedAt ?? null}
        onOpen={() => pane.open(code)}
        open={open}
        onOpenChange={onToggle}
        body={(
          <Flex direction="column" gap="sm">
            <Flex gap="xs" align="center" wrap>{signals}</Flex>
            {children}
          </Flex>
        )}
      />
    </Box>
  );
};

export { RunFrame };
