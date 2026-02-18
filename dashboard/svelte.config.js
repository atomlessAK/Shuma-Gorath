import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter({
      pages: '../dist/dashboard',
      assets: '../dist/dashboard',
      strict: true
    }),
    paths: {
      base: '/dashboard'
    },
    prerender: {
      crawl: false,
      entries: ['/', '/login.html']
    }
  }
};

export default config;
