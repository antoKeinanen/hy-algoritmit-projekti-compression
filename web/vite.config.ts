import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  // The wasm package loads its .wasm file relative to its own module, which
  // breaks if Vite pre-bundles it
  optimizeDeps: { exclude: ["wasm"] },
  // The linked wasm package lives outside this directory
  server: { fs: { allow: [".."] } },
});
