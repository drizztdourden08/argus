/* @layer renderer-app @kind component */
import { EmptyState, LogPanel } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useNow } from '../clock';
import { useProjectName } from '../projects';
import type { ActivityTabProps } from './activity.type';
import { NOTHING_MATCHES, NOTHING_YET, PANEL_HEIGHT } from './activity.constants';
import { useActivityFeed } from './behavior/useActivityFeed';
import { countLevels, kindDefs, levelOf, toLogRow } from './model';
import type { LevelFilter as Level } from './model';
import { LevelFilter } from './sub-components/LevelFilter';

const ActivityTab = (props: ActivityTabProps) => {
  const { freshSince } = props;
  const items = useActivityFeed();
  const nameOf = useProjectName();
  const now = useNow();
  const [level, setLevel] = useState<Level>('all');
  const kinds = useMemo(() => kindDefs(items), [items]);
  const counts = useMemo(() => countLevels(items), [items]);
  const rows = useMemo(
    () => items.filter((item) => level === 'all' || levelOf(item) === level).map((item) => toLogRow(item, { now, freshSince, nameOf })),
    [items, level, now, freshSince, nameOf],
  );
  if (items.length === 0) return <EmptyState message={NOTHING_YET} />;
  return (
    <LogPanel
      rows={rows}
      kinds={kinds}
      toolbarExtra={<LevelFilter value={level} counts={counts} onChange={setLevel} />}
      countLabel="entries"
      emptyLabel={NOTHING_MATCHES}
      height={PANEL_HEIGHT}
    />
  );
};

export { ActivityTab };
