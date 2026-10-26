/* @layer renderer-app @kind entry */
import '@drizztdourden08/tessera/tokens.css';
import '@drizztdourden08/tessera/palettes/archipelia.css';
import '@drizztdourden08/tessera/palettes/brock.css';
import '@drizztdourden08/tessera/palettes/bxel.css';
import '@drizztdourden08/tessera/palettes/rotp.css';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './routing/App';

const root = document.getElementById('root');
if (!root) throw new Error('index.html has no #root element');

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
