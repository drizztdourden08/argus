/* @layer renderer-app @kind logic */
import { useNavigate } from 'react-router';
import { useOpenEntry } from '../../../../shared';
import type { NodeOpens } from '../pane.type';

const useTrailGo = (open: (id: string) => void): ((target: NodeOpens) => void) => {
  const navigate = useNavigate();
  const openEntry = useOpenEntry();
  return (target) => {
    if (target.kind === 'board') void navigate(target.href);
    else if (target.kind === 'item') open(target.id);
    else if (target.kind === 'entry') openEntry(target.id);
    else void navigate({ hash: target.id });
  };
};

export { useTrailGo };
