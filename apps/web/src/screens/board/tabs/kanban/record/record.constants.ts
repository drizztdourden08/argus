/* @layer renderer-app @kind constants */
const ENTRY_KINDS = ['question', 'decision'];

const TIMELINE_KINDS = ['question', 'decision', 'followup', 'notice', 'message'];

const KIND_LABEL: Record<string, string> = {
  question: 'question',
  decision: 'decision',
  followup: 'note',
  notice: 'info',
  message: 'message',
  step: 'manual step',
  log: 'log',
};

const REPLY_KINDS = ['followup', 'decision'];

export { ENTRY_KINDS, KIND_LABEL, REPLY_KINDS, TIMELINE_KINDS };
