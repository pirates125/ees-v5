/** @type {import('next').NextConfig} */
const nextConfig = {
  // Development için standalone'i kaldırdık
  // Production deployment için gerekirse ekleyin: output: "standalone"
  experimental: {
    // Next.js 15 features
  },
};

module.exports = nextConfig;
