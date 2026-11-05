/* @layer renderer-app @kind logic */
import { useFocusChoices } from './useFocusChoices';

const useFocusActive = (boardKey: string): boolean => {
  const { focus, choices } = useFocusChoices(boardKey);
  return choices.options.some((option) => option.value === focus.focus);
};

export { useFocusActive };
