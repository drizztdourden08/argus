/* @layer renderer-app @kind logic */
import { ATTENTION_OF } from '../attention/attention.constants';
import type { AskLink, AskTagSpec } from './model.type';

const askTag = (link: AskLink): AskTagSpec => {
  const { icon, color } = ATTENTION_OF[link.kind];
  return link.kind === 'question'
    ? { id: link.id, label: `waiting on ${link.id}`, color, icon, title: `${link.id} is an open question, waiting for an answer` }
    : { id: link.id, label: `${link.id} to pick up`, color, icon, title: `${link.id} is a decision to pick up` };
};

export { askTag };
