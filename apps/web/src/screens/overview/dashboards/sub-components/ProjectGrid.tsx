/* @layer renderer-app @kind component */
import { EmptyState, Grid } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useNow } from '../../clock';
import { useNarrowWindow } from '../behavior/useNarrowWindow';
import { CARD_COLUMN_MIN, NARROW_COLUMNS, NO_PROJECT } from '../dashboards.constants';
import type { ProjectGridProps } from '../dashboards.type';
import { sortCards } from '../model';
import { ProjectCard } from './ProjectCard';

const ProjectGrid = (props: ProjectGridProps) => {
  const { cards, send, onLog } = props;
  const now = useNow();
  const narrow = useNarrowWindow();
  const sorted = useMemo(() => sortCards(cards), [cards]);
  if (cards.length === 0) return <EmptyState message={NO_PROJECT} />;
  return (
    <Grid {...(narrow ? { columns: NARROW_COLUMNS } : { minColWidth: CARD_COLUMN_MIN })} gap="md">
      {sorted.map((card) => <ProjectCard key={card.key} card={card} now={now} send={send} onLog={onLog} />)}
    </Grid>
  );
};

export { ProjectGrid };
