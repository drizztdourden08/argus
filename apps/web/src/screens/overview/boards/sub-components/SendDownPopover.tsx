/* @layer renderer-app @kind component */
import { Anchored, Card, Flex, Icon, IconButton, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { BoardMarkdownEditor } from '../../../../shell';
import { SEND_KINDS, SEND_MESSAGE } from '../boards-send.constants';
import type { SendDownPopoverProps } from '../boards.type';
import { OWNER_MAX, sendProblem } from '../model';
import type { SendKind } from '../model';
import './SendDownPopover.css';

const SendDownPopover = (props: SendDownPopoverProps) => {
  const { send, owner, anchorRef } = props;
  if (send.target !== owner) return null;
  const over = send.text.length > OWNER_MAX ? sendProblem(send.text) : null;
  return (
    <Anchored anchorRef={anchorRef} placement="bottom-start" onDismiss={send.close} className="send-down" role="dialog" aria-label={`Send down to ${send.name}`}>
      <Card>
        <Flex direction="column" gap="sm">
          <Flex gap="sm" align="center">
            <SegmentedControl<SendKind> aria-label="Kind of post" value={send.kind} options={SEND_KINDS} onChange={send.setKind} />
            <Text variant="caption" tone="muted" className="send-down__to">for <Text as="span" variant="caption" weight={600}>{send.name}</Text></Text>
            <IconButton label="Close" size="xs" variant="ghost" onClick={send.close}><Icon name="x" /></IconButton>
          </Flex>
          <BoardMarkdownEditor
            aria-label="Text to send down"
            compact
            preview={false}
            autoFocus
            placeholder={SEND_MESSAGE}
            value={send.text}
            onChange={send.setText}
            onSubmit={send.send}
            submitLabel="Send down"
            error={over}
            submitDisabled={over !== null}
          />
        </Flex>
      </Card>
    </Anchored>
  );
};

export { SendDownPopover };
