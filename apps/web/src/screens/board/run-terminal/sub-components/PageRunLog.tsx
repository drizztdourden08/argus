/* @layer renderer-app @kind component */
import { useCollection } from '../../../../data/react';
import type { RunDoc } from '../../../../data/types';
import type { PageRunLogProps } from '../run-terminal.type';
import { RunLogDrawer } from './RunLogDrawer';

const PageRunLog = (props: PageRunLogProps) => {
  const { boardKey, id, onClose } = props;
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  return <RunLogDrawer run={runs.docs.find((run) => run.id === id) ?? null} boardKey={boardKey} onClose={onClose} />;
};

export { PageRunLog };
