/* @layer renderer-app @kind constants */
import { defineStatuses, type ChatTranscriptRole } from '@drizztdourden08/tessera';
import type { LimitKey, LimitSource, OverlordMode, OverlordState } from '../../data/overlord';
import type { ChatTab, QueueSectionName } from './behavior/chat-panels.type';
import type { TalkKind, ToolStatus } from './OverlordChat.type';

const CHAT_PARAM = 'chat';
const CHAT_LABEL = 'Overlord chat';
const STARTING_CHAT = 'The Overlord is starting.';
const RUNNING_WORD = 'Running';
const NO_KEY_PATTERN = /no .*key/i;
const KEY_SETTINGS_LABEL = 'Set the key';
const CHAT_PLACEHOLDER = 'Message the Overlord';
const WRITING_LABEL = 'Writing';
const TAIL_MAX = 90;
const FINAL_EVENTS: readonly string[] = ['assistant', 'user', 'result'];
const EMPTY_TRANSCRIPT = 'Nothing has been said yet.';
const TALK_WHO: Readonly<Record<TalkKind, string>> = {
  assistant: 'Overlord',
  owner: 'Owner',
  wake: 'Wake',
  rule: 'Rule',
  thinking: 'Overlord',
  turn: 'Overlord',
  approval: 'Approval',
};
const TALK_ROLE: Readonly<Record<TalkKind, ChatTranscriptRole>> = {
  assistant: 'assistant',
  owner: 'user',
  wake: 'system',
  rule: 'system',
  thinking: 'assistant',
  turn: 'system',
  approval: 'system',
};
const SUMMARY_MAX = 120;
const SUMMARY_FIELDS = ['command', 'file_path', 'path', 'pattern', 'url', 'query'] as const;
const THINKING_LABEL = 'Thinking';
const QUEUED_NOTE = 'Queued, sent when the current step ends';
const SENDING_NOTE = 'Sending...';
const SENT_NOTE = 'Sent';
const APPROVAL_WORDS: Readonly<Record<string, string>> = { pending: 'waiting for an answer', allowed: 'allowed', denied: 'denied', expired: 'expired' };

const TOOL_STATUS = {
  running: { label: 'Running', tone: 'info', pulse: true },
  done: { label: 'Ran', tone: 'success' },
  failed: { label: 'Failed', tone: 'danger' },
} as const satisfies Record<ToolStatus, { label: string; tone: string; pulse?: boolean }>;
const TOOL_COUNT_WORDS = { done: 'done', failed: 'failed', running: 'running' } as const;
const TOOL_NAMES = { one: 'tool', many: 'tools' } as const;
const ERROR_CLAMP_LINES = 2;
const SETTING_LABELS = { model: 'Model', effort: 'Effort', permissionMode: 'Permission mode' } as const;
const SWITCHING_NOTE = 'Restarting the Overlord';
const USAGE_LABEL = 'Context and usage';
const USAGE_TITLES = { context: 'Context', session: 'This session', lastTurn: 'Last turn', month: 'This month' } as const;
const USAGE_WORDS = {
  context: 'Context used',
  model: 'Model',
  input: 'Input',
  output: 'Output',
  cacheRead: 'Cache read',
  cacheWrite: 'Cache write',
  cost: 'Cost',
  turns: 'Turns',
  duration: 'Duration',
  spent: 'Spent',
  budget: 'Budget',
  noBudget: 'None',
} as const;

const RULES_POLL_MS = 5_000;
const MIN_EVERY_MIN = 5;
const MAX_EVERY_MIN = 1440;
const TABS_LABEL = 'Overlord views';
const CHAT_TABS: { value: ChatTab; label: string }[] = [
  { value: 'chat', label: 'Chat' },
  { value: 'rules', label: 'Rules' },
  { value: 'queue', label: 'Queue' },
  { value: 'settings', label: 'Settings' },
];
const SETTINGS_ROWS: Readonly<Record<LimitKey, { title: string; note: string }>> = {
  cwd: { title: 'Working folder', note: 'The folder the Overlord works in.' },
  exePath: { title: 'Agent program path', note: 'The program that runs the Overlord.' },
  budgetUsd: { title: 'Monthly budget', note: 'Spend cap in USD, warned at 80 percent, paused at 100. Reset means no limit.' },
  restartMax: { title: 'Restarts allowed', note: 'How often it may restart on its own.' },
  approvalTimeoutMin: { title: 'Approval wait', note: 'Minutes to wait for an approval.' },
};
const SETTINGS_SECTIONS = { agent: 'Agent', limits: 'Limits' } as const;
const SOURCE_CAPTIONS: Readonly<Record<LimitSource, string>> = {
  set: 'Set in config.json',
  board: 'The folder of the Overlord board',
  default: 'Default',
};
const NO_RULES = 'No rules yet.';
const NONE_WAITING = 'None waiting.';
const QUEUE_EMPTY = 'Nothing is waiting for the Overlord.';
const QUEUE_TITLES: Readonly<Record<QueueSectionName, string>> = {
  wakes: 'Wakes',
  held: 'Held messages',
  alerts: 'Alerts',
};
const BLANK_DRAFT = { title: '', prompt: '', everyMin: 60 };

const OWNER_PREFIX = 'Owner (Argus Overlord tab): ';
const OWNER_SPLIT = /\n\n(?=Owner \(Argus Overlord tab\): )/;
const RUNNING_STATES: readonly OverlordState[] = ['idle', 'turn', 'waiting-approval'];

const STATE_STATUSES = defineStatuses({
  stopped: { label: 'Stopped', tone: 'neutral' },
  starting: { label: 'Starting', tone: 'info', pulse: true },
  idle: { label: 'Idle', tone: 'success' },
  turn: { label: 'Working', tone: 'info', pulse: true },
  'paused-budget': { label: 'Paused, budget reached', tone: 'warning' },
  'waiting-approval': { label: 'Waiting for approval', tone: 'warning' },
  crashed: { label: 'Crashed', tone: 'danger' },
});

const NO_LIMIT = 'No limit';
const READING = 'Reading the Overlord.';
const SPEND_NOTE = 'An estimate from token counts. The provider console has the real bill.';
const NO_MODE = 'Hosted, API key';

const MODE_NOTES: Readonly<Record<OverlordMode, string>> = {
  off: 'The desktop Overlord runs alone.',
  shadow: 'A read-only copy runs on your API key and logs what it would do.',
  live: 'The hosted Overlord takes over and the desktop one ends its turn.',
};
const MODE_CONFIRM = 'This moves the Overlord thread. The desktop Overlord and the hosted one swap places.';

const APPROVAL_PARAM = 'approval';

export {
  APPROVAL_PARAM,
  MODE_CONFIRM,
  MODE_NOTES,
  NO_LIMIT,
  NO_MODE,
  READING,
  SPEND_NOTE,
  STATE_STATUSES,
  APPROVAL_WORDS,
  ERROR_CLAMP_LINES,
  SETTING_LABELS,
  SWITCHING_NOTE,
  TOOL_COUNT_WORDS,
  TOOL_NAMES,
  TOOL_STATUS,
  USAGE_LABEL,
  USAGE_TITLES,
  USAGE_WORDS,
  QUEUED_NOTE,
  SENDING_NOTE,
  SENT_NOTE,
  SUMMARY_FIELDS,
  SUMMARY_MAX,
  THINKING_LABEL,
  WRITING_LABEL,
  TAIL_MAX,
  FINAL_EVENTS,
  BLANK_DRAFT,
  CHAT_LABEL,
  CHAT_PARAM,
  CHAT_PLACEHOLDER,
  CHAT_TABS,
  SETTINGS_ROWS,
  SETTINGS_SECTIONS,
  SOURCE_CAPTIONS,
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
