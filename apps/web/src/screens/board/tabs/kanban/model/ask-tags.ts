/* @layer renderer-app @kind logic */
import type { AskLink, AskTagSpec } from './model.type';

const askTag = (link: AskLink): AskTagSpec =>
  link.kind === 'question'
    ? { id: link.id, label: `waiting on ${link.id}`, color: 'warning', title: `${link.id} is an open question, waiting for an answer` }
    : { id: link.id, label: `${link.id} to pick up`, color: 'info', title: `${link.id} is a decision to pick up` };

export { askTag };
