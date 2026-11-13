/* @layer renderer-app @kind component */
import { BackdropLayer, Box } from '@drizztdourden08/tessera';
import { Suspense } from 'react';
import { Outlet } from 'react-router';
import { AppSettingsProvider } from '../AppSettings';
import { MeshBackdrop } from '../backdrop';
import { LoadingFallback } from '../LoadingFallback';
import { PageFrame } from '../PageFrame';
import { ShellBar } from '../ShellBar';
import { usePreloadPages } from './behavior/usePreloadPages';
import { useMigrateStored } from './behavior/useMigrateStored';
import { useShellMode } from './behavior/useShellMode';
import './AppShell.css';

const AppShell = () => {
  const { mode, boardKey, boardsAvailable, select } = useShellMode();
  useMigrateStored();
  usePreloadPages();
  return (
    <AppSettingsProvider>
      <Box className="app-shell">
        <BackdropLayer className="app-shell__backdrop" backdrop={{ kind: 'node', node: <MeshBackdrop /> }} />
        <ShellBar mode={mode} boardKey={boardKey} boardsAvailable={boardsAvailable} onMode={select} />
        <PageFrame>
          <Suspense fallback={<LoadingFallback />}>
            <Outlet />
          </Suspense>
        </PageFrame>
      </Box>
    </AppSettingsProvider>
  );
};

export { AppShell };
