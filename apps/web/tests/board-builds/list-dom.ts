/* @layer renderer-app @kind test */
import { defDoc } from './fixtures';

type Doc = Record<string, unknown>;

const withDefs = (launchers: Doc[], extra: object = {}) => ({
  launchers,
  launches: launchers.filter((launcher) => !launcher.retiredAt).map((launcher) => defDoc(String(launcher.name))),
  ...extra,
});

const cardOf = (container: HTMLElement, name: string): HTMLElement => container.querySelector<HTMLElement>(`[data-launcher="${name}"]`) as HTMLElement;

export { cardOf, withDefs };
