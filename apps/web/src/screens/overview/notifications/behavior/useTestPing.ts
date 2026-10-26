/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { sendTest } from '../../../../data/notify';
import { apiSentence, testSentence } from '../model';
import type { TestPing } from '../notifications.type';

const useTestPing = (): TestPing => {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');

  const send = useCallback(() => {
    if (busy) return;
    setBusy(true);
    setMessage('Sending...');
    sendTest()
      .then(
        (result) => setMessage(testSentence(result)),
        (failure: unknown) => setMessage(apiSentence(failure, (detail) => `Not sent: ${detail}.`)),
      )
      .finally(() => setBusy(false));
  }, [busy]);

  return { busy, message, send };
};

export { useTestPing };
