/* @layer renderer-app @kind component */
import { Flex, Icon, Link, Status, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { activityTone } from '../../shared';
import { LEAD_STATUS_TONES, NO_LEAD } from '../BoardHeader.constants';
import type { LeadStatusProps } from '../BoardHeader.type';

const LeadStatus = (props: LeadStatusProps) => {
  const { lead } = props;
  if (!lead) return <Text variant="caption" tone="muted">{NO_LEAD}</Text>;
  const tone = LEAD_STATUS_TONES[lead.status];
  return (
    <Flex gap="sm" align="center" wrap>
      <Status tone={activityTone(lead.activity)} pulse={lead.activity === 'working'} dot title={lead.activity ?? 'unknown'}>
        <Text weight={600} title={lead.session ?? lead.id}>{lead.name}</Text>
      </Status>
      {tone ? <Tag variant="urgency" color={tone} fill="tint">{lead.status}</Tag> : <Tag>{lead.status}</Tag>}
      <Text variant="caption" tone="muted">
        {lead.lastSeen ? <>seen <Age value={lead.lastSeen} ago /></> : 'never seen'}
      </Text>
      {lead.link ? (
        <Link href={lead.link} variant="subtle" aria-label="Open the chat of the lead" title="Open the lead chat">
          <Icon name="message-square" size={14} />
        </Link>
      ) : null}
    </Flex>
  );
};

export { LeadStatus };
