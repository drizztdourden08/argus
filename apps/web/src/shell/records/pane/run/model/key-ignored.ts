/* @layer renderer-app @kind logic */
import { TYPING_SELECTOR } from '../../../../../screens/board/tabs/kanban/kanban.constants';

const keyIgnored = (event: KeyboardEvent, target: Element | null): boolean =>
  Boolean(target?.closest(TYPING_SELECTOR)) || event.ctrlKey || event.metaKey || event.altKey || event.repeat;

export { keyIgnored };
