/* @layer renderer-app @kind component */
import { Flex, Pressable, Status, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import { ChatButton, activityTone, useOpenThread } from '../../../shared';
import { NO_NOTE_LINE } from '../AgentsTab.constants';
import type { LeadLineProps } from '../AgentsTab.type';
import { orText } from '../model';

const LeadLine = (props: LeadLineProps) => {
  const { lead, now } = props;
  const open = useOpenThread();
  const session = orText(lead.session, lead.id);
  const seen = lead.lastSeen ? `${formatAgo(lead.lastSeen, now)} ago` : 'never';
  return (
    <Flex className="agents-lead" align="center" gap="sm">
      <Text as="span" variant="caption" title={session}>lead</Text>
      <Status tone={activityTone(lead.activity)} dot title={`${session}: ${orText(lead.activity, 'unknown')}`} aria-label={orText(lead.activity, 'unknown')}>{null}</Status>
      <Pressable className="agents-lead__note" title={`${orText(lead.note, NO_NOTE_LINE)}\nSeen: ${seen}`} onClick={() => open(lead.id)}>
        {orText(lead.note, NO_NOTE_LINE)}
      </Pressable>
      <Text as="span" variant="caption" title={orText(lead.lastSeen, 'never seen')}>{`seen ${formatAgo(lead.lastSeen, now)}`}</Text>
      {lead.link && <ChatButton link={lead.link} label="Open the chat of the lead" />}
    </Flex>
  );
};

export { LeadLine };
