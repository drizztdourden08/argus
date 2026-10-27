/* @layer renderer-app @kind component */
import { Card, Flex, Spacer } from '@drizztdourden08/tessera';
import { attentionOf, cardTone, leadState } from '../model';
import type { ProjectCardProps } from '../dashboards.type';
import { CardCounters } from './CardCounters';
import { CardFlags } from './CardFlags';
import { CardFooter } from './CardFooter';
import { CardHeadline } from './CardHeadline';
import { CardState } from './CardState';
import { MasterRow } from './MasterRow';
import './ProjectCard.css';

const ProjectCard = (props: ProjectCardProps) => {
  const { card, now, onSend, onLog } = props;
  const state = leadState(card, now);
  const started = state !== 'not started';
  return (
    <Card fill className="project-card" data-state={state} data-edge={started ? cardTone(attentionOf(card)) : undefined}>
      <Flex direction="column" gap="sm">
        <CardHeadline card={card} state={state} />
        <CardState card={card} state={state} now={now} />
        {started && <CardCounters card={card} />}
        {started && <CardFlags card={card} />}
        {started && <MasterRow card={card} onLog={onLog} />}
        <Spacer />
        <CardFooter card={card} started={started} onSend={onSend} />
      </Flex>
    </Card>
  );
};

export { ProjectCard };
