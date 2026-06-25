// daisyUI + anime.js starter — interaction & motion layer.
//
// Classic script (no ESM): vendor/anime.umd.js is loaded just before this file
// and exposes the global `window.anime` with all named members (anime.js v4).
// This loads under `default-src 'self'` even at the opaque origin of the SBFB
// sandbox (no CORS-mode module fetch). No network, no worker, no fetch.
const { animate, stagger } = window.anime;

const REDUCE = matchMedia("(prefers-reduced-motion: reduce)").matches;

// Entrance: stagger the `.reveal` elements in. Skipped when the user prefers
// reduced motion — the content is already visible, the animation only adds it.
if (!REDUCE) {
  animate(".reveal", {
    opacity: [0, 1],
    translateY: [12, 0],
    delay: stagger(90),
    duration: 520,
    ease: "outQuad",
  });
}

// On click: pulse the daisyUI badge and drive the progress bar with anime.js.
const button = document.getElementById("pulse");
const badge = document.getElementById("badge");
const bar = document.getElementById("bar");

if (button) {
  button.addEventListener("click", () => {
    badge.textContent = "animé";
    if (REDUCE) {
      bar.value = 100;
      return;
    }
    animate(badge, {
      scale: [1, 1.35, 1],
      duration: 600,
      ease: "inOutQuad",
    });
    animate(bar, {
      value: [0, 100],
      duration: 900,
      ease: "outCubic",
    });
  });
}
