/* @layer root-config @kind config */
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const SERVER = 'http://127.0.0.1:47615';

export default defineConfig({
  plugins: [react()],
  resolve: { dedupe: ['react', 'react-dom'] },
  server: {
    proxy: {
      '/api': { target: SERVER, changeOrigin: true },
      '/mcp': { target: SERVER, changeOrigin: true },
      '/files': { target: SERVER, changeOrigin: true },
    },
  },
});
