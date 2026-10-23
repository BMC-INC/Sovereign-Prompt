/** @type {import('next').NextConfig} */
const nextConfig = {
  basePath: '/sovereign-prompt',
  // This text-only site does not need a server-side image optimizer.
  images: { unoptimized: true },
};

export default nextConfig;
