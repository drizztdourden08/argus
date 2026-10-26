/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';

const copyText = (text: string): void => {
  const clipboard = typeof navigator === 'undefined' ? undefined : navigator.clipboard;
  if (!clipboard) {
    toast({ message: 'Copy refused; select the text instead.', variant: 'warning' });
    return;
  }
  clipboard.writeText(text).then(
    () => toast({ message: 'Copied.', variant: 'success' }),
    () => toast({ message: 'Copy refused; select the text instead.', variant: 'warning' }),
  );
};

export { copyText };
