import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// `base` relativo: o site vai para o GitHub Pages num subcaminho
// (/Postly/), e caminho absoluto quebraria todo asset lá.
export default defineConfig({
  plugins: [react()],
  base: "./",
});
