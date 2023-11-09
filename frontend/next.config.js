/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: false,
    output: 'standalone',

    // eslint-disable-next-line @typescript-eslint/require-await
    async headers() {
        return [
            {
                source: '/',
                headers: [
                    {
                        key: 'Content-Security-Policy',
                        value: 'frame-ancestors http://ffxiv.quilan.io',
                    }
                ]
            }
        ];
    }
}

module.exports = nextConfig
