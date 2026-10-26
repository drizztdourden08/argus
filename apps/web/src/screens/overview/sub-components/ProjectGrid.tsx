/* @layer renderer-app @kind component */
import { EmptyState, Grid } from '@drizztdourden08/tessera';
import type { ProjectGridProps } from '../overview.type';
import { ProjectCard } from './ProjectCard';

const ProjectGrid = (props: ProjectGridProps) => {
  const { cards } = props;
  if (cards.length === 0) return <EmptyState message="No project has a board yet." />;
  return (
    <Grid minColWidth={240} gap="md">
      {cards.map((card) => <ProjectCard key={card.key} card={card} />)}
    </Grid>
  );
};

export { ProjectGrid };
