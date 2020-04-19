importScripts(
  "https://storage.googleapis.com/workbox-cdn/releases/4.3.1/workbox-sw.js"
);

self.addEventListener("install", event => {
  // Take over old service worker immediately, should hopefully fix weird caching issues
  self.skipWaiting();
});

// https://developers.google.com/web/fundamentals/instant-and-offline/offline-cookbook/#stale-while-revalidate
// TLDR: If there's a cached version available, use it, but fetch an update for next time.
const staleWhileRevalidate = new workbox.strategies.StaleWhileRevalidate();

// Remote fonts and JavaScript libraries
workbox.routing.registerRoute(
  new RegExp("https://fonts.googleapis.com/css"),
  staleWhileRevalidate
);
workbox.routing.registerRoute(
  new RegExp("https://fonts.gstatic.com"),
  staleWhileRevalidate
);
workbox.routing.registerRoute(
  new RegExp("https://maxcdn.bootstrapcdn.com/font-awesome"),
  staleWhileRevalidate
);
workbox.routing.registerRoute(
  new RegExp("https://cdnjs.cloudflare.com/ajax/libs/mathjax"),
  staleWhileRevalidate
);
workbox.routing.registerRoute(
  new RegExp("https://cdn.jsdelivr.net/clipboard.js"),
  staleWhileRevalidate
);

// Local resources
workbox.routing.registerRoute(
  /\.(woff2?|ttf|css|js|json|png|svg)(\?v\=.*)?$/,
  staleWhileRevalidate
);

// Here hbs_renderer.rs will inject the chapters, making sure they are precached.
//
// const chapters = [
//     { url: '/', revision: '11120' },
//     { url: 'cli/cli-tool.html', revision: '12722' },
//     { url: 'cli/init.html', revision: '12801' },
// ];
//
//   workbox.precaching.precacheAndRoute(chapters);
