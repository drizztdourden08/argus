/* @layer renderer-app @kind component */
import { Card, Flex, Spacer } from '@drizztdourden08/tessera';
import { SyncStamp } from '../../../../shell';
import { attentionOf, cardTone, leadState } from '../model';
import type { ProjectCardProps } from '../dashboards.type';
import { CardCounters } from './CardCounters';
import { CardFlags } from './CardFlags';
import { CardFooter } from './CardFooter';
import { CardHeadline } from './CardHeadline';
import { CardStalls } from './CardStalls';
import { CardState } from './CardState';
import { CardWorktrees } from './CardWorktrees';
import { MasterRow } from './MasterRow';
import './ProjectCard.css';

const ProjectCard = (props: ProjectCardProps) => {
  const { card, now, send, onLog, syncs } = props;
  const state = leadState(card, now);
  const started = state !== 'not started';
  return (
    <Card fill className="project-card" edge="border" edgeTone={started ? cardTone(attentionOf(card)) : undefined} data-state={state}>
      <Flex direction="column" gap="sm">
        <CardHeadline card={card} state={state} />
        <CardState card={card} state={state} now={now} />
        {started && syncs ? <SyncStamp sync={syncs[card.key] ?? null} /> : null}
        {started && <CardCounters card={card} />}
        {started && <CardWorktrees card={card} />}
        {started && <CardFlags card={card} />}
        {started && <CardStalls card={card} now={now} />}
        {started && <MasterRow card={card} onLog={onLog} />}
        <Spacer />
        <CardFooter card={card} started={started} send={send} />
      </Flex>
    </Card>
  );
};

export { ProjectCard };
