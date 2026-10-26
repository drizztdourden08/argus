/* @layer renderer-app @kind component */
import { Card, Link, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { resolvePalette } from '../../../palette';
import type { ProjectCardProps } from '../overview.type';

const ProjectCard = (props: ProjectCardProps) => {
  const { card } = props;
  const navigate = useNavigate();
  return (
    <Card variant="interactive" title={card.name} subtitle={card.key} data-palette={resolvePalette(card.key, card.palette)}>
      <Text variant="caption" tone="muted">{card.state}</Text>
      <Link href={card.url} navigate={navigate}>Open board</Link>
    </Card>
  );
};

export { ProjectCard };
