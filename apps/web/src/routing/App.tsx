/* @layer renderer-app @kind component */
import { TesseraProvider, ToastStack } from '@drizztdourden08/tessera';
import { BrowserRouter } from 'react-router';
import { DataProvider } from '../data/react';
import { MeshBackdrop } from '../shell/backdrop';
import { AppRoutes } from './AppRoutes';

const App = () => (
  <TesseraProvider>
    <MeshBackdrop />
    <DataProvider>
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </DataProvider>
    <ToastStack />
  </TesseraProvider>
);

export { App };
