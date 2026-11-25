/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { Age, AssigneeMark, RecordFrame, ShownCode } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import { RUN_KIND } from '../tests.constants';
import type { RunFrameProps } from '../tests-head-props.type';

const RunFrame = (props: RunFrameProps) => {
  const { view, muted = false, open, onToggle, menu, signals, children, ...look } = props;
  const { boardKey, threads } = useTestsShared();
  const { item } = view;
  const title = item.title || view.plan?.title;
  return (
    <RecordFrame
      {...look}
      name={item.id}
      kind={RUN_KIND}
      density="row"
      open={open}
      onOpenChange={onToggle}
      attrs={{ 'data-swap-item': '', 'data-run': item.id }}
      code={<Text font="title" tone={muted ? 'muted' : undefined} className="run-card__tag"><ShownCode id={item.id} /></Text>}
      title={title && <Text smartTimes clamp={1} tone={muted ? 'muted' : 'dim'} className="run-card__title">{title}</Text>}
      content={(
        <Flex gap="xs" align="center">
          {signals}
          {item.updatedAt && <Text as="span" variant="caption" leading="16" tone="muted"><Age value={item.updatedAt} /></Text>}
          <AssigneeMark assignee={item.assignee ?? item.thread ?? ''} boardKey={boardKey} threads={threads} />
        </Flex>
      )}
      menu={menu}
    >
      {children}
    </RecordFrame>
  );
};

export { RunFrame };
