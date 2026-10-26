/* @layer renderer-app @kind logic */
import type { Needs } from './model.type';

const needsYou = (needs: Needs | undefined): boolean => Boolean(needs && (needs.questions || needs.todos || needs.tests));

export { needsYou };
