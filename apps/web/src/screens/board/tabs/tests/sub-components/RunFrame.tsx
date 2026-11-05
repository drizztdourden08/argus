/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { RecordFrame, ShownCode } from '../../../../../shell';
import { RUN_KIND } from '../tests.constants';
import type { RunFrameProps } from '../tests-head-props.type';
import { RunChips } from './RunChips';

const RunFrame = (props: RunFrameProps) => {
  const { view, muted = false, open, onToggle, content, children } = props;
  const title = view.item.title || view.plan?.title;
  return (
    <RecordFrame
      name={view.item.id}
      kind={RUN_KIND}
      density="row"
      open={open}
      onOpenChange={onToggle}
      code={<Text font="title" tone={muted ? 'muted' : undefined} className="run-card__tag"><ShownCode id={view.item.id} /></Text>}
      title={title && <Text smartTimes tone={muted ? 'muted' : 'dim'} className="run-card__title">{title}</Text>}
      pinTitle
      content={
        <>
          <RunChips view={view} />
          {content}
        </>
      }
    >
      {children}
    </RecordFrame>
  );
};

export { RunFrame };
