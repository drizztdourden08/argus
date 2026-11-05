/* @layer renderer-app @kind test */
const NOW = '2026-10-26T08:00:00.000Z';

const base = { author: 'coordinator', thread: 'coordinator', status: 'open', createdAt: NOW };

const decision = {
  ...base,
  id: 'Q-1',
  kind: 'question',
  item: 'I-4',
  needs: 'decision',
  text: 'Create the private repo?',
  options: ['Yes, private', 'Not yet, stay local'],
  context: 'Para one.\n\nPara two.\n\nPara three.',
};

const freeText = { ...base, id: 'Q-2', kind: 'question', text: 'Which name?', context: 'A reason that is long enough to read.' };

const noContext = { ...base, id: 'Q-3', kind: 'question', text: 'Is this clear?' };

const notice = { ...base, id: 'N-1', kind: 'notice', text: 'Crash restored', level: 'warn' };

const message = { ...base, id: 'M-1', kind: 'message', text: 'Owner resumed the thread' };

const manualStep = { id: 'H-1', text: 'Restart the app', status: 'pending', thread: 'coordinator', item: 'I-4', createdAt: NOW };

const dismissal = (target: string, id = 'O-9') => ({
  id,
  kind: 'ack',
  author: 'human',
  replyTo: target,
  status: 'new',
  text: '',
  createdAt: NOW,
});

const byId = (...docs: ({ id: string } & Record<string, unknown>)[]): Record<string, object> =>
  Object.fromEntries(docs.map((doc) => [doc.id, doc]));

export { NOW, byId, decision, dismissal, freeText, manualStep, message, noContext, notice };
