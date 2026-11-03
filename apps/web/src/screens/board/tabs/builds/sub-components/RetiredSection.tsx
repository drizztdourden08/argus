/* @layer renderer-app @kind component */
import { Disclosure } from '@drizztdourden08/tessera';
import type { RetiredSectionProps } from '../builds-props.type';
import { LauncherList } from './LauncherList';

const RetiredSection = (props: RetiredSectionProps) => {
  const { views, flag, boardKey, runs, onLog } = props;
  if (views.length === 0) return null;
  return (
    <Disclosure open={flag.shown} onOpenChange={flag.set} summary={`Closed (${views.length})`}>
      <LauncherList views={views} label="Closed builds" boardKey={boardKey} runs={runs} onLog={onLog} />
    </Disclosure>
  );
};

export { RetiredSection };
