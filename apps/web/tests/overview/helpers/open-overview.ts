/* @layer renderer-app @kind test */
import { renderAt } from '../../helpers/render-at';
import { DEPS } from './deps';
import { feed } from './feed';
import { CARDS } from './fixtures';

const openOverview = async (location: string, deps: Record<string, object> = DEPS) => {
  const view = renderAt(location);
  await feed(view.sources, { cards: CARDS, deps });
  return view;
};

export { openOverview };
