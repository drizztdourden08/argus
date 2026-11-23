/* @layer renderer-app @kind logic */
import type { SettingChoice } from '../OverlordChat.type';

const labelOf = (choices: readonly SettingChoice[], id: string): string => choices.find((choice) => choice.id === id)?.label ?? id;

export { labelOf };
