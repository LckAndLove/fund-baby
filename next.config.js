/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  reactCompiler: true,
};

if (process.env.FUND_BABY_DESKTOP === '1') {
  nextConfig.output = 'export';
  nextConfig.images = {
    unoptimized: true,
  };
}

module.exports = nextConfig;

