import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8766",
        headers: { "X-Gen3-Token": process.env.GEN3_DEV_TOKEN ?? "" },
      },
    },
  },
  build: { outDir: "dist", sourcemap: false },
});
