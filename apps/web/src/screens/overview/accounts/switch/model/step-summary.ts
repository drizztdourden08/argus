/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { DETECT_NONE } from '../switch.constants';
import type { StageView } from '../switch.type';

const failureText = (doc: AccountsSwitch, view: StageView): string => (view.stage === 'sync' ? doc.sync?.error : undefined) ?? doc.error ?? view.text;

const detectText = (doc: AccountsSwitch, view: StageView): string => {
  const count = doc.threads?.length;
  if (count === undefined || view.state === 'waiting') return view.text;
  return count === 0 ? DETECT_NONE : `${count} working ${count === 1 ? 'thread' : 'threads'} found.`;
};

const stepSummary = (doc: AccountsSwitch, view: StageView): string => {
  if (view.state === 'failed') return failureText(doc, view);
  const text = view.stage === 'detect' ? detectText(doc, view) : view.text;
  return view.at ? `${text} ${view.at}` : text;
};

export { stepSummary };
