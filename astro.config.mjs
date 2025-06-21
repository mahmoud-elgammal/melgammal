import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import compress from "astro-compress";
import partytown from "@astrojs/partytown";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  site: "http://melgammal.com",
  integrations: [
    mdx(),
    sitemap(),
    compress(),
    partytown({
      config: {
        forward: ["dataLayer.push"],
      },
    }),
  ],
  vite: {
    plugins: [tailwindcss()],
  },
});
