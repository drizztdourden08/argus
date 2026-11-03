/* @layer renderer-app @kind logic */
import { useNavigate } from 'react-router';
import type { NodeOpens } from '../pane.type';

const useTrailGo = (open: (id: string) => void): ((target: NodeOpens) => void) => {
  const navigate = useNavigate();
  return (target) => {
    if (target.kind === 'board') void navigate(target.href);
    else if (target.kind === 'test') void navigate({ hash: target.id });
    else open(target.id);
  };
};

export { useTrailGo };
