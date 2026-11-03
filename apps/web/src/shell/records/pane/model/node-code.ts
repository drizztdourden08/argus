/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../data/registry';
import { ITEM_FALLBACK_KIND } from '../../behavior/record-grammar.constants';
import { shownCode } from '../../behavior/shown-code';
import type { TrailNode } from '../trail/trail.type';

const nodeCode = (node: TrailNode): string => {
  const kind = kindOf(node.type ?? ITEM_FALLBACK_KIND);
  return kind ? shownCode(kind, node.item) : node.item;
};

export { nodeCode };
