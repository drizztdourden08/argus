/* @layer renderer-app @kind logic */
import type { SelectState } from '../review.type';

const selectState = (ids: readonly string[], picked: ReadonlySet<string>): SelectState => {
  const count = ids.filter((id) => picked.has(id)).length;
  const all = ids.length > 0 && count === ids.length;
  return { checked: all, mixed: count > 0 && !all };
};

export { selectState };
