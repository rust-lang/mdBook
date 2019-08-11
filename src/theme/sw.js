importScripts(
  "https://unpkg.com/workbox-sw@2.0.3/build/importScripts/workbox-sw.dev.v2.0.3.js"
);

// clientsClaims tells the Service Worker to take control as soon as it's activated
const workbox = new WorkboxSW({ clientsClaim: true });

// https://developers.google.com/web/fundamentals/instant-and-offline/offline-cookbook/#stale-while-revalidate
// TLDR: If there's a cached version available, use it, but fetch an update for next time.
const staleWhileRevalidate = workbox.strategies.staleWhileRevalidate();

// Remote fonts and JavaScript libraries
workbox.router.registerRoute(
  new RegExp("https://fonts.googleapis.com/css"),
  staleWhileRevalidate
);
workbox.router.registerRoute(
  new RegExp("https://fonts.gstatic.com"),
  staleWhileRevalidate
);
workbox.router.registerRoute(
  new RegExp("https://maxcdn.bootstrapcdn.com/font-awesome"),
  staleWhileRevalidate
);
workbox.router.registerRoute(
  new RegExp("https://cdnjs.cloudflare.com/ajax/libs/mathjax"),
  staleWhileRevalidate
);
workbox.router.registerRoute(
  new RegExp("https://cdn.jsdelivr.net/clipboard.js"),
  staleWhileRevalidate
);

// Local resources
workbox.router.registerRoute(new RegExp(".js$"), staleWhileRevalidate);
workbox.router.registerRoute(new RegExp(".css$"), staleWhileRevalidate);

// Here hbs_renderer.rs will inject the chapters, making sure they are precached.
//
// const chapters = [
//     { url: '/', revision: '11120' },
//     { url: 'cli/cli-tool.html', revision: '12722' },
//     { url: 'cli/init.html', revision: '12801' },
// ];
//
//   workbox.precaching.precacheAndRoute(chapters);
