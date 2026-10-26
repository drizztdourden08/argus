/* @layer renderer-app @kind component */
import { Button, DateText, Flex, Link, Status, Text } from '@drizztdourden08/tessera';
import { activityTone, useOpenThread } from '../../../shared';
import type { LeadLineProps } from '../AgentsTab.type';
import { orText } from '../model';

const LeadLine = (props: LeadLineProps) => {
  const { lead } = props;
  const open = useOpenThread();
  return (
    <Flex gap="sm" align="center" wrap>
      <Text tone="muted" title={orText(lead.session, lead.id)}>lead</Text>
      <Status tone={activityTone(lead.activity)} dot>{orText(lead.activity, 'unknown')}</Status>
      <Button size="xs" variant="ghost" title="Open the lead thread" onClick={() => open(lead.id)}>{orText(lead.note, 'no note')}</Button>
      <Text tone="muted">
        seen {lead.lastSeen ? <DateText value={lead.lastSeen} format="relative" /> : 'never'}
      </Text>
      {lead.link && <Link href={lead.link} variant="subtle" external>Open the lead chat</Link>}
    </Flex>
  );
};

export { LeadLine };
