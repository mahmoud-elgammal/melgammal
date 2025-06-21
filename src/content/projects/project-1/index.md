---
title: "Syriazone"
description: "Portfolio and blog build with astro."
date: "Mar 18 2024"
# demoURL: "https://astro-sphere-demo.vercel.app"
# repoURL: "https://github.com/markhorn-dev/astro-sphere"
---

Syriazone (http://syriazone.sy/) is a cutting-edge news aggregator that transforms how users consume news by delivering curated, region-specific content with a focus on speed, reliability, and user experience. As the sole full-stack developer and team lead, I architected and built Syriazone to address the need for a seamless platform that aggregates diverse news sources into a single, intuitive interface. This case study highlights the project’s journey, from conception to execution, with an emphasis on UX design and technical innovation.


### Project Goals
- Create a visually appealing, user-centric platform for news consumption.
- Aggregate real-time news from multiple sources with high reliability.
- Ensure lightning-fast performance and scalability for growing audiences.
- Build a maintainable, secure system as a solo developer.

## Challenges & Solutions

### Challenge: Diverse Data Formats
News sources provided inconsistent data (JSON, XML, etc.), complicating aggregation.

**Solution**: Developed a Rust-based parser to normalize data into a unified JSON schema, stored in PostgreSQL for consistency.

### Challenge: Solo Development Time Constraints
As the only developer, balancing frontend, backend, and UX tasks was demanding.

**Solution**: Adopted an agile approach with weekly sprints, prioritizing MVP features (news feed, search) before adding enhancements (filters, dark mode).

### Challenge: Performance Optimization
Initial load times were slightly higher than desired due to large API payloads.

**Solution**: Implemented lazy loading for images, compressed assets, and cached API responses, achieving a Lighthouse score of 97.


## Conclusion
Syriazone is a testament to the power of combining UX expertise with modern technology. As the sole full-stack developer and team lead, I delivered a platform that balances performance, scalability, and user delight. By prioritizing user needs and leveraging Next.js, Rust, PostgreSQL, and Tailwind CSS, Syriazone sets a new standard for news aggregation. This project showcases my ability to architect, build, and lead a complex web application from vision to victory.

## Results
- **Performance**: Achieved sub-1-second page loads and a Lighthouse score of 97 for performance, SEO, and accessibility.
- **Scalability**: Handled 15,000 concurrent users in stress tests, thanks to Rust’s efficiency and caching.
- **User Engagement**: Beta testers reported a 35% increase in session duration due to the intuitive UI and relevant content.
- **Reliability**: Maintained 99.95% uptime since launch, with zero critical bugs reported.
