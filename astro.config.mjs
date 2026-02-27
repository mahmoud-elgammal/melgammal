import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import compress from "astro-compress";
import partytown from "@astrojs/partytown";
// import cloudflare from "@astrojs/cloudflare";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";

export default defineConfig({
  site: "http://melgammal.com",
  output: "static",
  // adapter: cloudflare(),
  integrations: [
    mdx({
      remarkPlugins: [remarkMath],
      rehypePlugins: [rehypeKatex],
    }),
    sitemap(),
    compress(),
    partytown({
      config: {
        forward: ["dataLayer.push"],
      },
    }),
  ],
});
