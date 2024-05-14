// vite.config.ts
import { sveltekit } from "file:///home/leart/Applications/dev/cxsign_workplace/csm/node_modules/.pnpm/@sveltejs+kit@2.5.4_@sveltejs+vite-plugin-svelte@3.0.2_svelte@4.2.12_vite@5.2.6/node_modules/@sveltejs/kit/src/exports/vite/index.js";
import { vite as vidstack } from "file:///home/leart/Applications/dev/cxsign_workplace/csm/node_modules/.pnpm/vidstack@1.11.16/node_modules/vidstack/plugins.js";
import { defineConfig } from "file:///home/leart/Applications/dev/cxsign_workplace/csm/node_modules/.pnpm/vite@5.2.6/node_modules/vite/dist/node/index.js";
import { internalIpV4 } from "file:///home/leart/Applications/dev/cxsign_workplace/csm/node_modules/.pnpm/internal-ip@7.0.0/node_modules/internal-ip/index.js";
var mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);
var vite_config_default = defineConfig(async () => ({
  plugins: [vidstack({ include: /player\// }), sveltekit()],
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: mobile ? "0.0.0.0" : false,
    hmr: mobile ? {
      protocol: "ws",
      host: await internalIpV4(),
      port: 1421
    } : void 0,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"]
    }
  }
}));
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvaG9tZS9sZWFydC9BcHBsaWNhdGlvbnMvZGV2L2N4c2lnbl93b3JrcGxhY2UvY3NtXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvaG9tZS9sZWFydC9BcHBsaWNhdGlvbnMvZGV2L2N4c2lnbl93b3JrcGxhY2UvY3NtL3ZpdGUuY29uZmlnLnRzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy9ob21lL2xlYXJ0L0FwcGxpY2F0aW9ucy9kZXYvY3hzaWduX3dvcmtwbGFjZS9jc20vdml0ZS5jb25maWcudHNcIjtpbXBvcnQgeyBzdmVsdGVraXQgfSBmcm9tIFwiQHN2ZWx0ZWpzL2tpdC92aXRlXCI7XG5pbXBvcnQgeyB2aXRlIGFzIHZpZHN0YWNrIH0gZnJvbSBcInZpZHN0YWNrL3BsdWdpbnNcIjtcbmltcG9ydCB7IGRlZmluZUNvbmZpZyB9IGZyb20gXCJ2aXRlXCI7XG5pbXBvcnQgeyBpbnRlcm5hbElwVjQgfSBmcm9tIFwiaW50ZXJuYWwtaXBcIjtcblxuLy8gQHRzLWV4cGVjdC1lcnJvciBwcm9jZXNzIGlzIGEgbm9kZWpzIGdsb2JhbFxuY29uc3QgbW9iaWxlID0gISEvYW5kcm9pZHxpb3MvLmV4ZWMocHJvY2Vzcy5lbnYuVEFVUklfRU5WX1BMQVRGT1JNKTtcblxuLy8gaHR0cHM6Ly92aXRlanMuZGV2L2NvbmZpZy9cbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyhhc3luYyAoKSA9PiAoe1xuICBwbHVnaW5zOiBbdmlkc3RhY2soeyBpbmNsdWRlOiAvcGxheWVyXFwvLyB9KSwgc3ZlbHRla2l0KCldLFxuXG4gIC8vIFZpdGUgb3B0aW9ucyB0YWlsb3JlZCBmb3IgVGF1cmkgZGV2ZWxvcG1lbnQgYW5kIG9ubHkgYXBwbGllZCBpbiBgdGF1cmkgZGV2YCBvciBgdGF1cmkgYnVpbGRgXG4gIC8vXG4gIC8vIDEuIHByZXZlbnQgdml0ZSBmcm9tIG9ic2N1cmluZyBydXN0IGVycm9yc1xuICBjbGVhclNjcmVlbjogZmFsc2UsXG4gIC8vIDIuIHRhdXJpIGV4cGVjdHMgYSBmaXhlZCBwb3J0LCBmYWlsIGlmIHRoYXQgcG9ydCBpcyBub3QgYXZhaWxhYmxlXG4gIHNlcnZlcjoge1xuICAgIHBvcnQ6IDUxNzMsXG4gICAgc3RyaWN0UG9ydDogdHJ1ZSxcbiAgICBob3N0OiBtb2JpbGUgPyBcIjAuMC4wLjBcIiA6IGZhbHNlLFxuICAgIGhtcjogbW9iaWxlXG4gICAgICA/IHtcbiAgICAgICAgICBwcm90b2NvbDogXCJ3c1wiLFxuICAgICAgICAgIGhvc3Q6IGF3YWl0IGludGVybmFsSXBWNCgpLFxuICAgICAgICAgIHBvcnQ6IDE0MjEsXG4gICAgICAgIH1cbiAgICAgIDogdW5kZWZpbmVkLFxuICAgIHdhdGNoOiB7XG4gICAgICAvLyAzLiB0ZWxsIHZpdGUgdG8gaWdub3JlIHdhdGNoaW5nIGBzcmMtdGF1cmlgXG4gICAgICBpZ25vcmVkOiBbXCIqKi9zcmMtdGF1cmkvKipcIl0sXG4gICAgfSxcbiAgfSxcbn0pKTtcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBcVUsU0FBUyxpQkFBaUI7QUFDL1YsU0FBUyxRQUFRLGdCQUFnQjtBQUNqQyxTQUFTLG9CQUFvQjtBQUM3QixTQUFTLG9CQUFvQjtBQUc3QixJQUFNLFNBQVMsQ0FBQyxDQUFDLGNBQWMsS0FBSyxRQUFRLElBQUksa0JBQWtCO0FBR2xFLElBQU8sc0JBQVEsYUFBYSxhQUFhO0FBQUEsRUFDdkMsU0FBUyxDQUFDLFNBQVMsRUFBRSxTQUFTLFdBQVcsQ0FBQyxHQUFHLFVBQVUsQ0FBQztBQUFBO0FBQUE7QUFBQTtBQUFBLEVBS3hELGFBQWE7QUFBQTtBQUFBLEVBRWIsUUFBUTtBQUFBLElBQ04sTUFBTTtBQUFBLElBQ04sWUFBWTtBQUFBLElBQ1osTUFBTSxTQUFTLFlBQVk7QUFBQSxJQUMzQixLQUFLLFNBQ0Q7QUFBQSxNQUNFLFVBQVU7QUFBQSxNQUNWLE1BQU0sTUFBTSxhQUFhO0FBQUEsTUFDekIsTUFBTTtBQUFBLElBQ1IsSUFDQTtBQUFBLElBQ0osT0FBTztBQUFBO0FBQUEsTUFFTCxTQUFTLENBQUMsaUJBQWlCO0FBQUEsSUFDN0I7QUFBQSxFQUNGO0FBQ0YsRUFBRTsiLAogICJuYW1lcyI6IFtdCn0K
