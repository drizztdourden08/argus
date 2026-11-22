/* @layer renderer-app @kind constants */
import type { ChatTranscriptRole } from '@drizztdourden08/tessera';
import type { OverlordState } from '../../data/overlord';
import type { ChatTab, QueueSectionName } from './OverlordChat.type';

const CHAT_PARAM = 'chat';
const CHAT_LABEL = 'Overlord chat';
const STARTING_CHAT = 'The Overlord is starting.';
const RUNNING_WORD = 'Running';
const NO_KEY_PATTERN = /no .*key/i;
const KEY_SETTINGS_LABEL = 'Set the key';
const CHAT_PLACEHOLDER = 'Message the Overlord. It reads as Owner (Argus Overlord tab).';
const EMPTY_TRANSCRIPT = 'Nothing has been said yet.';
const TALK_WHO: Readonly<Record<'assistant' | 'owner' | 'wake', string>> = { assistant: 'Overlord', owner: 'Owner', wake: 'Wake' };
const TALK_ROLE: Readonly<Record<'assistant' | 'owner' | 'wake', ChatTranscriptRole>> = { assistant: 'assistant', owner: 'user', wake: 'system' };

const RULES_POLL_MS = 5_000;
const MIN_EVERY_MIN = 5;
const MAX_EVERY_MIN = 1440;
const TABS_LABEL = 'Overlord views';
const CHAT_TABS: { value: ChatTab; label: string }[] = [
  { value: 'chat', label: 'Chat' },
  { value: 'rules', label: 'Rules' },
  { value: 'queue', label: 'Queue' },
];
const NO_RULES = 'No rules yet.';
const NONE_WAITING = 'None waiting.';
const QUEUE_EMPTY = 'Nothing is waiting for the Overlord.';
const QUEUE_TITLES: Readonly<Record<QueueSectionName, string>> = {
  wakes: 'Wakes',
  held: 'Held messages',
  approvals: 'Approvals',
  alerts: 'Alerts',
};
const BLANK_DRAFT = { title: '', prompt: '', everyMin: 60 };

const OWNER_PREFIX = 'Owner (Argus Overlord tab): ';
const OWNER_SPLIT = /\n\n(?=Owner \(Argus Overlord tab\): )/;
const RUNNING_STATES: readonly OverlordState[] = ['idle', 'turn', 'waiting-approval'];

export {
  BLANK_DRAFT,
  CHAT_LABEL,
  CHAT_PARAM,
  CHAT_PLACEHOLDER,
  CHAT_TABS,
  MAX_EVERY_MIN,
  MIN_EVERY_MIN,
  NONE_WAITING,
  NO_RULES,
  QUEUE_EMPTY,
  QUEUE_TITLES,
  RULES_POLL_MS,
  TABS_LABEL,
  EMPTY_TRANSCRIPT,
  OWNER_PREFIX,
  OWNER_SPLIT,
  RUNNING_STATES,
  STARTING_CHAT,
  RUNNING_WORD,
  NO_KEY_PATTERN,
  KEY_SETTINGS_LABEL,
  TALK_ROLE,
  TALK_WHO,
};
