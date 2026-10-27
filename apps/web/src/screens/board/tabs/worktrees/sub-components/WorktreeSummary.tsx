/* @layer renderer-app @kind component */
import { DateText } from '@drizztdourden08/tessera';
import { summaryParts } from '../model';
import type { WorktreeSummaryProps } from '../worktrees-props.type';

const WorktreeSummary = (props: WorktreeSummaryProps) => {
  const { counts, meta } = props;
  if (!meta?.rules?.length) return null;
  const parts = summaryParts(counts, meta);
  const tail = [parts.gone, parts.problems, parts.many].filter(Boolean);
  return (
    <>
      {`${parts.counts} · ${parts.need} · `}
      {meta.scannedAt ? <>scanned <DateText value={meta.scannedAt} format="elapsed" length="short" /> ago</> : 'not scanned yet'}
      {tail.map((part) => ` · ${part}`)}
    </>
  );
};

export { WorktreeSummary };
