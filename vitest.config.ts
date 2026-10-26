/* @layer root-config @kind config */
import { defineConfig } from 'vitest/config';

export default defineConfig({ test: { projects: ['apps/web'] } });
