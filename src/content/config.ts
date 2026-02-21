import { defineCollection, z } from "astro:content";

const blog = defineCollection({
  type: "content",
  schema: z.object({
    title: z.string(),
    description: z.string(),
    date: z.coerce.date(),
    draft: z.boolean().optional(),
    image: z.string().optional(),
    color: z.string().optional(),
  }),
});

const work = defineCollection({
  type: "content",
  schema: z.object({
    company: z.string(),
    role: z.string(),
    dateStart: z.coerce.date(),
    dateEnd: z.union([z.coerce.date(), z.string()]),
  }),
});

const projects = defineCollection({
  type: "content",
  schema: z.object({
    title: z.string(),
    description: z.string(),
    date: z.coerce.date(),
    draft: z.boolean().optional(),
    demoURL: z.string().optional(),
    repoURL: z.string().optional(),
    tags: z.array(z.string()).optional(),
    imageWeb: z.string().optional(),
    imageMobile: z.string().optional(),
    color: z.string().optional(),
    status: z.enum(["shipped", "in-progress", "concept"]).default("shipped"),
    role: z.string().optional(),
    duration: z.string().optional(),
    client: z.string().optional(),
    stats: z
      .array(
        z.object({
          value: z.string(),
          label: z.string(),
        }),
      )
      .optional(),
    timeline: z
      .array(
        z.object({
          phase: z.string(),
          date: z.string(),
          title: z.string(),
          description: z.string(),
        }),
      )
      .optional(),
    gallery: z
      .array(
        z.object({
          type: z.enum(["image", "ambient-video", "cinematic-video"]).default("image").optional(),
          src: z.string(),
          alt: z.string().optional(),
          poster: z.string().optional(), // Used for video poster frames
          caption: z.string().optional(),
        }),
      )
      .optional(),
    inlineMedia: z
      .array(
        z.object({
          type: z.enum(["image", "ambient-video", "cinematic-video"]).default("image").optional(),
          src: z.string(),
          alt: z.string().optional(),
          poster: z.string().optional(),
          caption: z.string().optional(),
        }),
      )
      .optional(),
  }),
});

const playground = defineCollection({
  type: "content",
  schema: z.object({
    title: z.string(),
    description: z.string(),
    date: z.coerce.date(),
    draft: z.boolean().optional(),
    demoURL: z.string().optional(),
    repoURL: z.string().optional(),
    tags: z.array(z.string()).optional(),
    image: z.string().optional(),
    color: z.string().optional(),
    author: z.string().optional(),
  }),
});

export const collections = { blog, work, projects, playground };
