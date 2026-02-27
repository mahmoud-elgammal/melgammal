---
title: "Syriazone"
description: "A cutting-edge news aggregator delivering curated, region-specific content with lightning performance and seamless UX."
date: "Mar 18 2024"
# demoURL: "https://astro-sphere-demo.vercel.app"
# repoURL: "https://github.com/markhorn-dev/astro-sphere"
tags: ["Astro", "Rust", "PostgreSQL", "Next.js"]
imageWeb: "../../../assets/projects/syriazone_web.png"
imageMobile: "../../../assets/projects/syriazone_mobile.png"
color: "#8B5CF6"
status: "shipped"
role: "Full-Stack Developer & Team Lead"
duration: "4 months"
client: "Internal Product"
stats:
  - value: "97"
    label: "Lighthouse Score"
  - value: "15K+"
    label: "Concurrent Users"
  - value: "99.95%"
    label: "Uptime"
  - value: "<1s"
    label: "Page Load"
timeline:
  - phase: "Discovery"
    date: "Dec 2023"
    title: "Research & Architecture"
    description: "Analyzed 20+ news APIs, designed the unified data schema, and mapped out the system architecture for real-time aggregation."
  - phase: "Design"
    date: "Jan 2024"
    title: "UX & Visual System"
    description: "Built a component design system, prototyped the reading experience, and validated with user testing sessions."
  - phase: "Development"
    date: "Feb 2024"
    title: "Core Platform Build"
    description: "Developed the Rust-based parser for multi-format data normalization, Next.js frontend, and PostgreSQL data layer."
  - phase: "Launch"
    date: "Mar 2024"
    title: "Deployment & Optimization"
    description: "Deployed to production, optimized performance to sub-1s loads, achieved Lighthouse 97, and stress-tested to 15K concurrent users."
gallery:
  - type: "cinematic-video"
    src: "https://videos.pexels.com/video-files/3129595/3129595-uhd_2560_1440_30fps.mp4"
    poster: "../../../assets/projects/syriazone_web.png"
    alt: "Syriazone Data Pipeline Demo"
    caption: "Real-time content aggregation running"
  - type: "image"
    src: "../../../assets/projects/syriazone_web.png"
    alt: "Syriazone homepage"
    caption: "News feed with real-time aggregation"
  - type: "image"
    src: "../../../assets/projects/syriazone_mobile.png"
    alt: "Syriazone mobile experience"
    caption: "Responsive mobile reading experience"
inlineMedia:
  - type: "ambient-video"
    src: "https://videos.pexels.com/video-files/3129595/3129595-uhd_2560_1440_30fps.mp4"
    poster: "../../../assets/projects/syriazone_web.png"
    caption: "Inline Ambient Demonstration (Hover to VIEW)"
  - type: "cinematic-video"
    src: "https://videos.pexels.com/video-files/3129595/3129595-uhd_2560_1440_30fps.mp4"
    poster: "../../../assets/projects/syriazone_mobile.png"
    caption: "Stand-alone Cinematic Player"
---

Syriazone is a cutting-edge news aggregator that transforms how users consume news by delivering curated, region-specific content with a focus on speed, reliability, and user experience. As the sole full-stack developer and team lead, I architected and built the entire platform from the ground up.

Challenge: Normalizing inconsistent data from diverse news sources (JSON, XML, RSS) into a unified schema. I solved this by building a custom Rust-based parser that processes feeds in real-time, storing normalized data in PostgreSQL.

The frontend was built with Next.js for optimal performance, featuring lazy loading, compressed assets, and aggressive caching — achieving a Lighthouse score of 97 and sub-1-second page loads. This System Design choice was critical for high-traffic scalability.

The platform now This real-time regional news with near-zero latency, transforming how users interact with curated content across the Middle East. Despite the Risk of data fragmentation, the final architecture remains robust and reliable.
