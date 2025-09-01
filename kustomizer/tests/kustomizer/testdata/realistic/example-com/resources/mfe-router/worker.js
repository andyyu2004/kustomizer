export default {
  async fetch(req, env, ctx) {
    return new Response('Hello from Cloudflare Worker!', { status: 200 });
  },
};
