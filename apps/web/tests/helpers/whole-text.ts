/* @layer renderer-app @kind test */
const wholeText = (expected: RegExp) => (_content: string, element: Element | null): boolean => {
  if (!element || !expected.test(element.textContent)) return false;
  return !Array.from(element.children).some((child) => expected.test(child.textContent));
};

export { wholeText };
