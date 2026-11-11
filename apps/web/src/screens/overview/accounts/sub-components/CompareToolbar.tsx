/* @layer renderer-app @kind component */
import { Button, Checkbox, Inline } from '@drizztdourden08/tessera';
import type { CompareToolbarProps } from '../compare.type';

const CompareToolbar = (props: CompareToolbarProps) => {
  const { filter, busy, onFilter, onBack, onRescan, onPlan, onSync } = props;
  return (
    <Inline gap="md" align="center" wrap>
      <Button size="sm" variant="ghost" onClick={onBack}>Back</Button>
      <Checkbox label="Show same rows" checked={filter.showSame} onChange={(showSame) => onFilter({ ...filter, showSame })} />
      <Checkbox label="Show archived" checked={filter.showArchived} onChange={(showArchived) => onFilter({ ...filter, showArchived })} />
      <Button size="sm" variant="ghost" disabled={busy} onClick={onRescan}>Rescan</Button>
      <Button size="sm" variant="ghost" disabled={busy} onClick={onPlan}>Plan</Button>
      <Button size="sm" disabled={busy} onClick={onSync}>Sync now</Button>
    </Inline>
  );
};

export { CompareToolbar };
