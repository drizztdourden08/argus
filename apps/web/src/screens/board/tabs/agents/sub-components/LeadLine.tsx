/* @layer renderer-app @kind component */
import { DateText, Flex, Link, Status, Text } from '@drizztdourden08/tessera';
import type { LeadLineProps } from '../AgentsTab.type';
import { activityTone, orText } from '../model';

const LeadLine = (props: LeadLineProps) => {
  const { lead } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text tone="muted" title={orText(lead.session, lead.id)}>lead</Text>
      <Status tone={activityTone(lead.activity)} dot>{orText(lead.activity, 'unknown')}</Status>
      <Text>{orText(lead.note, 'no note')}</Text>
      <Text tone="muted">
        seen {lead.lastSeen ? <DateText value={lead.lastSeen} format="relative" /> : 'never'}
      </Text>
      {lead.link && <Link href={lead.link} variant="subtle" external>Open the lead chat</Link>}
    </Flex>
  );
};

export { LeadLine };
