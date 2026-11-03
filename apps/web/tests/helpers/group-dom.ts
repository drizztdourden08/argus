/* @layer renderer-app @kind test */
const groupOf = (container: HTMLElement, id: string): HTMLElement => {
  const found = container.querySelector<HTMLElement>(`[data-group="${id}"]`);
  if (!found) throw new Error(`no group ${id}`);
  return found;
};

const order = (container: HTMLElement): (string | null)[] => [...container.querySelectorAll('[data-group]')].map((group) => group.getAttribute('data-group'));

export { groupOf, order };
