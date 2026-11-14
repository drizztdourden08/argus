/* @layer renderer-app @kind constants */
import type { OverlordState } from '../../data/overlord';

const CHAT_PARAM = 'chat';
const CHAT_LABEL = 'Overlord chat';
const STOPPED_CHAT = 'The Overlord is stopped. Start it to send a message.';
const CHAT_PLACEHOLDER = 'Message the Overlord. It reads as Owner (Argus Overlord tab).';
const EMPTY_TRANSCRIPT = 'Nothing has been said yet.';
const TALK_WHO: Readonly<Record<'assistant' | 'owner' | 'wake', string>> = { assistant: 'Overlord', owner: 'Owner', wake: 'Wake' };

const OWNER_PREFIX = 'Owner (Argus Overlord tab): ';
const OWNER_SPLIT = /\n\n(?=Owner \(Argus Overlord tab\): )/;
const RUNNING_STATES: readonly OverlordState[] = ['idle', 'turn', 'waiting-approval'];

export {
  CHAT_LABEL,
  CHAT_PARAM,
  CHAT_PLACEHOLDER,
  EMPTY_TRANSCRIPT,
  OWNER_PREFIX,
  OWNER_SPLIT,
  RUNNING_STATES,
  STOPPED_CHAT,
  TALK_WHO,
};
