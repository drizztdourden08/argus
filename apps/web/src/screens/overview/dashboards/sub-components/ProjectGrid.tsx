/* @layer renderer-app @kind component */
import { EmptyState, Grid } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useNow } from '../../clock';
import { NO_PROJECT } from '../dashboards.constants';
import type { ProjectGridProps } from '../dashboards.type';
import { sortCards } from '../model';
import { ProjectCard } from './ProjectCard';

const ProjectGrid = (props: ProjectGridProps) => {
  const { cards } = props;
  const now = useNow();
  const sorted = useMemo(() => sortCards(cards), [cards]);
  if (cards.length === 0) return <EmptyState message={NO_PROJECT} />;
  return (
    <Grid minColWidth={260} gap="md">
      {sorted.map((card) => <ProjectCard key={card.key} card={card} now={now} />)}
    </Grid>
  );
};

export { ProjectGrid };
