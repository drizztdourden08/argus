/* @layer renderer-app @kind entry */
import '@drizztdourden08/tessera/tokens.css';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Text } from '@drizztdourden08/tessera';

const root = document.getElementById('root');
if (!root) throw new Error('index.html has no #root element');

createRoot(root).render(
  <StrictMode>
    <Text variant="title">Argus</Text>
  </StrictMode>,
);
