/* @layer renderer-app @kind component */
import { Button, Card, CopyButton, Inline, Link, Stack } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { resolvePalette } from '../../../../palette';
import type { ProjectCardProps } from '../dashboards.type';
import { attentionOf, cardTone, leadState } from '../model';
import { CardCounters } from './CardCounters';
import { CardFlags } from './CardFlags';
import { CardHeadline } from './CardHeadline';
import { MasterRow } from './MasterRow';

const ProjectCard = (props: ProjectCardProps) => {
  const { card, now, onSend } = props;
  const navigate = useNavigate();
  const started = leadState(card, now) !== 'not started';
  return (
    <Card
      variant="interactive"
      title={card.name}
      subtitle={card.key}
      tone={started ? cardTone(attentionOf(card)) : undefined}
      data-palette={resolvePalette(card.key, card.palette)}
    >
      <Stack gap="sm">
        <CardHeadline card={card} now={now} />
        {started ? <CardCounters card={card} /> : null}
        {started ? <CardFlags card={card} /> : null}
        {started ? <MasterRow card={card} now={now} /> : null}
        <Inline gap="sm" align="center">
          {started ? <Link href={card.url} navigate={navigate}>Open board</Link> : null}
          {started ? <Button size="xs" onClick={() => onSend(card.key)} title={`Send a decision or a note to ${card.name}`}>Send down</Button> : null}
          <CopyButton text={card.key} label="Copy the project key" size="xs" />
        </Inline>
      </Stack>
    </Card>
  );
};

export { ProjectCard };
