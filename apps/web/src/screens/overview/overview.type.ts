/* @layer renderer-app @kind types */
import type { Card } from '../../data/types';

interface ProjectCardProps {
  card: Card;
}

interface ProjectGridProps {
  cards: Card[];
}

export type { ProjectCardProps, ProjectGridProps };
