// daisyUI x anime.js showcase — interaction & motion layer.
// Classic script (no ESM): anime.js v4 is loaded just before this file as a
// vendored UMD bundle that exposes the global `anime`. This is the pattern
// every shipped SBFB app uses, and it loads under `default-src 'self'` even in
// the opaque-origin sandbox (no CORS-mode module fetch). No network, no worker.
const {
  animate,
  stagger,
  createTimeline,
  utils,
  createSpring,
  createAnimatable,
  createDraggable,
  spring,
  svg,
  splitText,
  scrambleText,
} = window.anime;

const REDUCE = matchMedia("(prefers-reduced-motion: reduce)").matches;

// View Transitions helper — degrades to a plain DOM mutation when the API
// is missing or the user prefers reduced motion.
const withViewTransition = (fn) => {
  if (REDUCE || !document.startViewTransition) return void fn();
  document.startViewTransition(fn);
};

// ───────────────────────── Hero : titre lettre par lettre ─────────────────
(() => {
  const title = document.querySelector(".hero-title");
  if (!title) return;
  const text = title.textContent.trim();
  title.setAttribute("aria-label", text);
  title.textContent = "";
  for (const ch of text) {
    const span = document.createElement("span");
    span.className = "char";
    span.setAttribute("aria-hidden", "true");
    span.textContent = ch === " " ? " " : ch;
    title.appendChild(span);
  }
  if (REDUCE) {
    utils.set(".hero-title .char, .hero-in", { opacity: 1 });
    return;
  }
  animate(".hero-title .char", {
    y: [42, 0],
    opacity: [0, 1],
    scale: [0.9, 1],
    duration: 850,
    delay: stagger(34),
    ease: "outExpo",
  });
  animate(".hero-in", {
    opacity: [0, 1],
    y: [18, 0],
    delay: stagger(90, { start: 420 }),
    duration: 650,
    ease: "outQuad",
  });
})();

// ───────────────────────── Stats : compteurs au scroll ────────────────────
(() => {
  const countUp = (el) => {
    const to = parseFloat(el.dataset.to) || 0;
    if (REDUCE) return void (el.textContent = to.toLocaleString("fr-FR"));
    const obj = { v: 0 };
    animate(obj, {
      v: to,
      duration: 1500,
      ease: "outExpo",
      onUpdate: () => {
        el.textContent = Math.round(obj.v).toLocaleString("fr-FR");
      },
    });
  };
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        io.unobserve(e.target);
        e.target.querySelectorAll(".count").forEach(countUp);
      }
    },
    { threshold: 0.35 },
  );
  document.querySelectorAll("[data-countgroup]").forEach((g) => io.observe(g));
})();

// ───────────────────────── Design system : contenu généré ─────────────────
(() => {
  // Tokens Reflect (valeurs sources copiées dans l'app ; le thème daisyUI
  // utilise des équivalents oklch). Affichés tels quels comme référence.
  const palette = [
    ["Canvas", "#111111", "base-100"],
    ["Surface", "#191919", "base-200"],
    ["Surface +", "#2E2E2E", "base-300"],
    ["Texte", "#EBEBEB", "content"],
    ["Muté", "#999999", "—"],
    ["Discret", "#666666", "—"],
    ["Accent", "#EBEBEB", "primary"],
    ["Succès", "#4ADE80", "success"],
    ["Alerte", "#FACC15", "warning"],
    ["Danger", "#F87171", "error"],
  ];
  const pal = document.getElementById("palette");
  if (pal) {
    for (const [name, hex, role] of palette) {
      const cell = document.createElement("div");
      cell.className = "space-y-1";
      const sw = document.createElement("div");
      sw.className = "h-14 rounded-box border border-base-content/10";
      sw.style.background = hex;
      const label = document.createElement("div");
      label.className = "text-xs";
      label.innerHTML =
        `<div class="font-medium">${name}</div>` +
        `<div class="font-mono opacity-50">${hex}</div>` +
        `<div class="font-mono opacity-40">${role}</div>`;
      cell.append(sw, label);
      pal.appendChild(cell);
    }
  }

  const spacing = document.getElementById("spacing");
  if (spacing) {
    for (const px of [8, 16, 24, 32, 48, 64]) {
      const row = document.createElement("div");
      row.className = "flex items-center gap-3";
      const bar = document.createElement("div");
      bar.className = "h-4 rounded bg-primary/80";
      bar.style.width = px + "px";
      const lab = document.createElement("span");
      lab.className = "text-xs font-mono opacity-60";
      lab.textContent = px + "px";
      row.append(bar, lab);
      spacing.appendChild(row);
    }
  }

  const radii = document.getElementById("radii");
  if (radii) {
    for (const px of [2, 4, 8, 12, 16]) {
      const box = document.createElement("div");
      box.className = "size-14 bg-base-100 border border-base-content/20 grid place-items-end p-1";
      box.style.borderRadius = px + "px";
      const lab = document.createElement("span");
      lab.className = "text-[10px] font-mono opacity-60";
      lab.textContent = px;
      box.appendChild(lab);
      radii.appendChild(box);
    }
  }
})();

// ───────────────────────── Motion lab : grille anime.js ───────────────────
const grid = [14, 8];
(() => {
  const wave = document.getElementById("wave");
  if (!wave) return;
  const frag = document.createDocumentFragment();
  for (let i = 0; i < grid[0] * grid[1]; i++) frag.appendChild(document.createElement("i"));
  wave.appendChild(frag);
  if (REDUCE) return;
  createTimeline({ loop: true, defaults: { ease: "inOutSine" } })
    .add("#wave i", {
      scale: [1, 0.3],
      opacity: [0.9, 0.35],
      duration: 900,
      delay: stagger(80, { grid, from: "center" }),
    })
    .add(
      "#wave i",
      {
        scale: [0.3, 1],
        opacity: [0.35, 0.9],
        duration: 900,
        delay: stagger(80, { grid, from: "center" }),
      },
      "-=500",
    );
})();

const pulseWave = () => {
  if (REDUCE) return;
  animate("#wave i", {
    scale: [1, 1.5, 1],
    rotate: [0, 90, 0],
    duration: 1100,
    delay: stagger(18, { grid, from: "center" }),
    ease: "inOutQuad",
  });
};

const playMotionDemo = () => {
  document.querySelectorAll(".motion-dot").forEach((dot) => {
    if (REDUCE) return;
    const dur = parseInt(dot.dataset.dur, 10) || 120;
    animate(dot, { translateX: [0, 44, 0], duration: dur * 2, ease: "inOutQuad" });
  });
};

// Actions déclaratives via data-anim.
const ACTIONS = {
  "hero-cta": () => {
    document.getElementById("motion")?.scrollIntoView({ behavior: REDUCE ? "auto" : "smooth" });
    pulseWave();
  },
  pulse: pulseWave,
  burst: () => {
    if (REDUCE) return;
    animate("#wave i", {
      translateX: () => utils.random(-26, 26),
      translateY: () => utils.random(-26, 26),
      rotate: () => utils.random(-120, 120),
      scale: () => utils.random(5, 16) / 10,
      duration: 650,
      ease: "outBack", // overshoot ASSUMÉ : c'est le bac à sable « démo », pas un composant UI
      alternate: true,
      loop: 1,
    });
  },
  "motion-demo": playMotionDemo,
};
document.querySelectorAll("[data-anim]").forEach((el) =>
  el.addEventListener("click", () => ACTIONS[el.dataset.anim]?.()),
);

// ───────────────────────── Boutons magnétiques ────────────────────────────
(() => {
  if (REDUCE) return;
  document.querySelectorAll(".magnetic").forEach((btn) => {
    btn.addEventListener("pointermove", (e) => {
      const r = btn.getBoundingClientRect();
      animate(btn, {
        translateX: (e.clientX - (r.left + r.width / 2)) * 0.25,
        translateY: (e.clientY - (r.top + r.height / 2)) * 0.4,
        duration: 260,
        ease: "outQuad",
      });
    });
    btn.addEventListener("pointerleave", () => {
      animate(btn, {
        translateX: 0,
        translateY: 0,
        duration: 600,
        ease: spring({ stiffness: 120, damping: 12 }),
      });
    });
  });
})();

// ───────────────────────── Thème (View Transition) ────────────────────────
(() => {
  const select = document.getElementById("theme");
  if (!select) return;
  select.addEventListener("change", () => {
    withViewTransition(() =>
      document.documentElement.setAttribute("data-theme", select.value),
    );
  });
})();

// ───────────────────────── Galerie filtrable (View Transition + FLIP) ──────
(() => {
  const gallery = document.getElementById("gallery");
  const filters = document.getElementById("filters");
  if (!gallery || !filters) return;
  const cards = [...gallery.querySelectorAll("[data-cat]")];
  // Noms stables → le navigateur morphe les positions (FLIP) pendant la VT.
  cards.forEach((c, i) => (c.style.viewTransitionName = `gcard-${i}`));

  const apply = (cat) => {
    withViewTransition(() => {
      for (const c of cards) c.hidden = !(cat === "all" || c.dataset.cat === cat);
    });
  };
  filters.querySelectorAll("[data-filter]").forEach((tab) => {
    tab.addEventListener("click", () => {
      filters.querySelector(".tab-active")?.classList.remove("tab-active");
      tab.classList.add("tab-active");
      apply(tab.dataset.filter);
    });
  });
})();

// ───────────────────────── Panneau (@starting-style + allow-discrete) ──────
(() => {
  const modal = document.getElementById("modal");
  if (!modal) return;
  const open = () => (modal.hidden = false);
  const close = () => (modal.hidden = true);
  document.querySelectorAll("[data-open-modal]").forEach((b) => b.addEventListener("click", open));
  document.querySelectorAll("[data-close-modal]").forEach((b) => b.addEventListener("click", close));
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !modal.hidden) close();
  });
})();

// ───────────────────────── Atelier anime.js × daisyUI ─────────────────────
(() => {
  // 1) Carte 3D réactive au curseur (createAnimatable — efficace en flux).
  document.querySelectorAll(".tilt-card").forEach((card) => {
    if (REDUCE) return;
    const inner = card.querySelector(".tilt-card__inner");
    const glow = card.querySelector(".tilt-card__glow");
    if (!inner) return;
    const a = createAnimatable(inner, { rotateX: 320, rotateY: 320, ease: "out(3)" });
    const g = glow ? createAnimatable(glow, { x: 280, y: 280, ease: "out(3)" }) : null;
    card.addEventListener("pointermove", (e) => {
      const r = card.getBoundingClientRect();
      const px = (e.clientX - r.left) / r.width - 0.5;
      const py = (e.clientY - r.top) / r.height - 0.5;
      a.rotateY(px * 16);
      a.rotateX(-py * 16);
      if (g) {
        g.x(px * r.width * 0.6);
        g.y(py * r.height * 0.6);
      }
    });
    card.addEventListener("pointerleave", () => {
      a.rotateX(0);
      a.rotateY(0);
    });
  });

  // 2) Jauge radiale daisyUI pilotée par anime.js (anime → variable CSS --value).
  const runGauge = (el) => {
    const to = parseFloat(el.dataset.to) || 0;
    if (REDUCE) {
      el.style.setProperty("--value", to);
      el.textContent = `${to}%`;
      return;
    }
    const obj = { v: 0 };
    animate(obj, {
      v: to,
      duration: 1600,
      ease: "outExpo",
      onUpdate: () => {
        const n = Math.round(obj.v);
        el.style.setProperty("--value", n);
        el.textContent = `${n}%`;
      },
    });
  };
  document.querySelectorAll(".gauge").forEach((el) => {
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (!e.isIntersecting) continue;
          io.unobserve(e.target);
          runGauge(e.target);
        }
      },
      { threshold: 0.4 },
    );
    io.observe(el);
  });
  document.querySelectorAll("[data-gauge-replay]").forEach((b) =>
    b.addEventListener("click", () => document.querySelectorAll(".gauge").forEach(runGauge)),
  );

  // 3) Signature SVG (createDrawable) + texte scramble (anime onUpdate, 0 dep texte).
  const scramble = (el, target) => {
    if (REDUCE) return void (el.textContent = target);
    const pool = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789#@%&";
    const obj = { p: 0 };
    animate(obj, {
      p: 1,
      duration: 900,
      ease: "outQuad",
      onUpdate: () => {
        const shown = Math.floor(obj.p * target.length);
        let out = target.slice(0, shown);
        for (let i = shown; i < target.length; i++) {
          out += target[i] === " " ? " " : pool[Math.floor(Math.random() * pool.length)];
        }
        el.textContent = out;
      },
    });
  };
  const runSign = () => {
    const path = document.getElementById("sig-path");
    if (path && svg) {
      const drawable = svg.createDrawable(path);
      if (REDUCE) utils.set(drawable, { draw: "0 1" });
      else animate(drawable, { draw: ["0 0", "0 1"], duration: 1200, ease: "inOutSine" });
    }
    const text = document.getElementById("sig-text");
    if (text) scramble(text, "Signature verifiee");
  };
  document.querySelectorAll("[data-sign-run]").forEach((b) => b.addEventListener("click", runSign));

  // 4) Étiquettes draggables à ressort, contraintes au conteneur.
  if (!REDUCE && typeof createDraggable === "function") {
    document.querySelectorAll(".drag-zone").forEach((zone) => {
      zone.querySelectorAll(".drag-chip").forEach((chip) => {
        createDraggable(chip, {
          container: zone,
          releaseEase: spring({ stiffness: 160, damping: 14 }),
        });
      });
    });
  }

  // 5) Paquet qui suit une route (createMotionPath) — élément SVG, coords exactes.
  const dot = document.getElementById("route-dot");
  if (dot && !REDUCE && svg) {
    animate(dot, {
      ...svg.createMotionPath("#route"),
      duration: 3200,
      loop: true,
      ease: "inOutQuad",
    });
  }
})();

// ═══════════ SBFB — composants vivants (ancrés dans le code réel) ══════════

// 1) Échelle de vérification N0→N3 — miroir de criticality_maps_to_verification_level.
(() => {
  const root = document.getElementById("vc-ladder");
  if (!root) return;
  const steps = [...root.querySelectorAll("#vc-steps .step")];
  const verifiable = root.querySelector("#vc-verifiable");
  const redun = root.querySelector("#vc-redundancy");
  const out = root.querySelector("#vc-level");
  const names = ["N0", "N1", "N2", "N3"];
  // verification.rs: verifiable && redundancy>1 => N2 ; verifiable => N1 ; sinon N0. N3 jamais auto.
  const compute = () => (verifiable.checked ? (redun.checked ? 2 : 1) : 0);
  const apply = () => {
    const lvl = compute();
    out.textContent = names[lvl];
    out.className = `badge badge-lg ${lvl >= 2 ? "badge-success" : lvl === 1 ? "badge-warning" : "badge-ghost"}`;
    steps.forEach((s, i) => {
      if (i === 3) return; // N3 jamais dérivé de la criticité
      s.classList.toggle("step-primary", i <= lvl);
    });
    if (!REDUCE) {
      const lit = steps.filter((_, i) => i <= lvl && i < 3);
      if (lit.length) animate(lit, { opacity: [0.45, 1], translateX: [-6, 0], duration: 360, delay: stagger(70), ease: "outQuad" });
    }
  };
  [verifiable, redun].forEach((el) =>
    el.addEventListener("change", () => (REDUCE ? apply() : withViewTransition(apply))),
  );
  apply();
})();

// 2) Couverture du pipeline sharded — miroir is_pipeline_contiguous + covers_full_model.
(() => {
  const root = document.getElementById("sc-coverage");
  if (!root) return;
  const ribbon = root.querySelector("#sc-ribbon");
  const verdict = root.querySelector("#sc-verdict");
  const tabs = root.querySelector("#sc-fixtures");
  const TOTAL = 32;
  const FIX = {
    valide: [[0, 8], [8, 18], [18, 32]],
    trou: [[0, 8], [10, 18], [18, 32]],
    chevauchement: [[0, 10], [8, 18], [18, 32]],
    partiel: [[0, 8], [8, 20]],
  };
  const check = (asg) => {
    if (!asg.length) return { ok: false, fault: "plan vide", idx: -1 };
    const s = asg.map((a, i) => ({ a, i })).sort((x, y) => x.a[0] - y.a[0]);
    if (s[0].a[0] !== 0) return { ok: false, fault: "ne commence pas à 0", idx: s[0].i };
    for (let k = 0; k < s.length; k++) {
      if (s[k].a[1] <= s[k].a[0]) return { ok: false, fault: "bloc vide", idx: s[k].i };
      if (k > 0 && s[k].a[0] !== s[k - 1].a[1]) {
        const gap = s[k].a[0] > s[k - 1].a[1];
        return { ok: false, fault: gap ? `trou ${s[k - 1].a[1]}–${s[k].a[0]}` : `chevauchement à ${s[k].a[0]}`, idx: s[k].i };
      }
    }
    if (s[s.length - 1].a[1] !== TOTAL) return { ok: false, fault: `s'arrête à ${s[s.length - 1].a[1]} (≠ ${TOTAL})`, idx: s[s.length - 1].i };
    return { ok: true, idx: -1 };
  };
  const render = (key) => {
    const asg = FIX[key];
    const res = check(asg);
    ribbon.innerHTML = "";
    asg.forEach(([start, end], i) => {
      const b = document.createElement("div");
      b.className = "sc-block" + (!res.ok && res.idx === i ? " is-fault" : "");
      b.style.left = (start / TOTAL) * 100 + "%";
      b.style.width = ((end - start) / TOTAL) * 100 + "%";
      b.textContent = `S${i + 1}`;
      ribbon.appendChild(b);
      if (!REDUCE) animate(b, { scaleX: [0, 1], duration: 420, delay: i * 80, ease: "outExpo" });
    });
    verdict.textContent = res.ok ? "couvre le modèle ✓" : res.fault;
    verdict.className = "badge " + (res.ok ? "badge-success" : "badge-error");
    if (res.ok && !REDUCE) {
      const tok = document.createElement("div");
      tok.className = "sc-token";
      ribbon.appendChild(tok);
      animate(tok, { translateX: [0, ribbon.offsetWidth - 12], duration: 1500, ease: "inOutSine", loop: true, alternate: true });
    }
  };
  tabs.querySelectorAll("[data-fix]").forEach((t) =>
    t.addEventListener("click", () => {
      tabs.querySelector(".tab-active")?.classList.remove("tab-active");
      t.classList.add("tab-active");
      render(t.dataset.fix);
    }),
  );
  render("valide");
})();

// 3) Pouls de joignabilité — tri-état + fraîcheur TTL (browse.rs).
(() => {
  const list = document.getElementById("rp-list");
  if (!list) return;
  const INIT = [
    { name: "sbfb-explorer", state: "reachable", age: 12 },
    { name: "ideas-hub", state: "unknown", age: 0 },
    { name: "vieux-proto", state: "unreachable", age: 41 },
  ];
  let apps = INIT.map((a) => ({ ...a }));
  const LABEL = { reachable: "joignable", unreachable: "injoignable", unknown: "inconnu (cache froid)" };
  const render = () => {
    list.innerHTML = "";
    apps.forEach((app) => {
      const li = document.createElement("li");
      li.className = "flex items-center gap-3";
      const dot = document.createElement("span");
      const color = app.state === "reachable" ? "bg-success" : app.state === "unreachable" ? "bg-error" : "bg-base-content/30";
      dot.className = `size-2.5 rounded-full shrink-0 ${color}`;
      const txt = document.createElement("div");
      txt.className = "flex-1 min-w-0";
      txt.innerHTML = `<div class="text-sm truncate">${app.name}</div><div class="text-xs opacity-50">${LABEL[app.state]}${app.state === "reachable" ? ` · vérifié il y a ${app.age}s` : ""}</div>`;
      const bar = document.createElement("span");
      bar.className = "h-1 w-16 rounded-full bg-base-100 overflow-hidden shrink-0";
      const fill = document.createElement("span");
      fill.className = "block h-full bg-success origin-left";
      bar.appendChild(fill);
      li.append(dot, txt, bar);
      list.appendChild(li);
      if (REDUCE) {
        if (app.state !== "reachable") fill.style.transform = "scaleX(0)";
        return;
      }
      if (app.state === "reachable") {
        animate(dot, { opacity: [1, 0.35], scale: [1, 0.85], loop: true, alternate: true, duration: 1500, ease: "inOutSine" });
        animate(fill, { scaleX: [1, 0], duration: 6000, ease: "linear", onComplete: () => { app.state = "unknown"; render(); } });
      } else if (app.state === "unknown") {
        fill.style.transform = "scaleX(0)";
        animate(dot, { rotate: "1turn", loop: true, duration: 1100, ease: "linear" });
      } else {
        fill.style.transform = "scaleX(0)";
      }
    });
  };
  const btn = document.createElement("button");
  btn.className = "btn btn-xs btn-outline mt-3";
  btn.textContent = "Re-sonder";
  btn.addEventListener("click", () => { apps = INIT.map((a) => ({ ...a })); render(); });
  list.after(btn);
  const io = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { io.unobserve(e.target); render(); } }), { threshold: 0.3 });
  io.observe(list);
})();

// 4) Carte de preuve — score additif déterministe (proof_card.rs).
(() => {
  const root = document.getElementById("pc-proof");
  if (!root) return;
  const gauge = root.querySelector("#pc-gauge");
  const bar = root.querySelector("#pc-bar");
  const layersEl = root.querySelector("#pc-layers");
  const risksEl = root.querySelector("#pc-risks");
  const tabs = root.querySelector("#pc-fixtures");
  const CASES = {
    ideal: { provenance: true, oss: true, fresh: "fresh", curators: 3, license: true, archive: true, repo: true, risks: [] },
    risky: { provenance: false, oss: false, fresh: "stale", curators: 0, license: false, archive: true, repo: false, risks: ["no_provenance", "unverified_deploy", "stale_source"] },
  };
  const RISK_LABEL = { no_provenance: "provenance absente", unverified_deploy: "déploiement non vérifié", stale_source: "source ancienne", old_release: "version ancienne" };
  const RISK_PTS = { no_provenance: -15, unverified_deploy: -10, stale_source: -10, old_release: -5 };
  const build = (c) => {
    const s = [["base", 30, "bg-base-content/30"]];
    if (c.provenance) s.push(["provenance", 20, "bg-success"]);
    if (c.oss) s.push(["open-source", 10, "bg-success/70"]);
    if (c.fresh === "fresh") s.push(["fraîcheur", 10, "bg-info"]);
    else if (c.fresh === "aging") s.push(["fraîcheur", 5, "bg-info"]);
    if (c.curators >= 1) s.push(["curateur≥1", 10, "bg-primary"]);
    if (c.curators >= 3) s.push(["curateur≥3", 10, "bg-primary/70"]);
    if (c.license) s.push(["licence", 5, "bg-secondary"]);
    if (c.archive) s.push(["archive", 5, "bg-secondary/60"]);
    const penalty = c.risks.reduce((n, r) => n + (RISK_PTS[r] || 0), 0);
    const score = Math.max(0, Math.min(100, s.reduce((n, x) => n + x[1], 0) + penalty));
    return { segs: s, score };
  };
  const colorOf = (sc) => (sc >= 70 ? "text-success" : sc >= 40 ? "text-warning" : "text-error");
  const render = (key) => {
    const c = CASES[key];
    const { segs, score } = build(c);
    gauge.className = "radial-progress shrink-0 " + colorOf(score);
    if (REDUCE) { gauge.style.setProperty("--value", score); gauge.textContent = score; }
    else { const o = { v: 0 }; animate(o, { v: score, duration: 1500, ease: "outExpo", onUpdate: () => { const n = Math.round(o.v); gauge.style.setProperty("--value", n); gauge.textContent = n; } }); }
    bar.innerHTML = "";
    segs.forEach((seg, i) => {
      const d = document.createElement("span");
      d.className = "pc-seg " + seg[2];
      d.style.width = seg[1] + "%";
      d.title = `${seg[0]} +${seg[1]}`;
      bar.appendChild(d);
      if (!REDUCE) animate(d, { scaleX: [0, 1], duration: 500, delay: i * 70, ease: "outExpo" });
    });
    const L = [["Provenance", c.provenance], ["Licence", c.license], ["Fraîcheur", c.fresh === "fresh" || c.fresh === "aging"], ["Curation", c.curators >= 1], ["Archive", c.archive], ["Source", c.repo]];
    layersEl.innerHTML = "";
    L.forEach(([n, ok]) => {
      const li = document.createElement("li");
      li.className = "flex items-center gap-1 " + (ok ? "" : "opacity-40");
      li.innerHTML = `<span class="${ok ? "text-success" : "text-base-content/40"}">${ok ? "✓" : "·"}</span> ${n}`;
      layersEl.appendChild(li);
    });
    risksEl.innerHTML = "";
    if (!c.risks.length) {
      const b = document.createElement("span");
      b.className = "badge badge-sm badge-success badge-outline";
      b.textContent = "aucun risque";
      risksEl.appendChild(b);
    } else c.risks.forEach((r) => {
      const b = document.createElement("span");
      b.className = "badge badge-sm badge-error badge-outline";
      b.textContent = RISK_LABEL[r] || r;
      risksEl.appendChild(b);
      if (!REDUCE) animate(b, { translateX: [0, -3, 3, 0], duration: 420, ease: "inOutSine" });
    });
  };
  tabs.querySelectorAll("[data-case]").forEach((t) =>
    t.addEventListener("click", () => {
      tabs.querySelector(".tab-active")?.classList.remove("tab-active");
      t.classList.add("tab-active");
      render(t.dataset.case);
    }),
  );
  const io = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { io.unobserve(e.target); render("ideal"); } }), { threshold: 0.3 });
  io.observe(root);
})();

// 5) Constellation de redondance — Toi + N pairs (daemon.ts seedCount, best-effort).
(() => {
  const root = document.getElementById("sd-const");
  if (!root) return;
  const stage = root.querySelector("#sd-stage");
  const core = root.querySelector("#sd-core");
  const countEl = root.querySelector("#sd-count");
  const self = root.querySelector("#sd-self");
  let n = 3;
  let started = false;
  const ring = document.createElement("span");
  ring.className = "sd-ring";
  ring.style.width = "150px";
  ring.style.height = "150px";
  stage.appendChild(ring);
  const orbit = document.createElement("div");
  orbit.className = "sd-orbit";
  stage.appendChild(orbit);
  const layout = () => {
    orbit.innerHTML = "";
    const r = 75;
    for (let i = 0; i < n; i++) {
      const node = document.createElement("span");
      node.className = "sd-node";
      node.style.transform = `rotate(${(i / Math.max(1, n)) * 360}deg) translateX(${r}px)`;
      orbit.appendChild(node);
      if (!REDUCE) animate(node, { opacity: [0, 1], scale: [0, 1], duration: 500, delay: i * 60, ease: "outBack" });
    }
    countEl.textContent = `${n} pair${n > 1 ? "s" : ""}`;
  };
  core.classList.toggle("is-seeding", self.checked);
  self.addEventListener("change", () => core.classList.toggle("is-seeding", self.checked));
  root.querySelectorAll("[data-sd]").forEach((b) =>
    b.addEventListener("click", () => { n = Math.max(0, Math.min(6, n + parseInt(b.dataset.sd, 10))); layout(); }),
  );
  const io = new IntersectionObserver((es) => es.forEach((e) => {
    if (!e.isIntersecting) return;
    io.unobserve(e.target);
    layout();
    if (!REDUCE && !started) { started = true; animate(orbit, { rotate: "1turn", loop: true, duration: 26000, ease: "linear" }); }
  }), { threshold: 0.3 });
  io.observe(root);
})();

// 6) Cockpit contributeur GPU — consentement L1-L4 + caps + fit VRAM (consent.ts, QUANTIZATION.md).
(() => {
  const root = document.getElementById("gc-cockpit");
  if (!root) return;
  const range = root.querySelector("#gc-range");
  const steps = [...root.querySelectorAll("#gc-levels .step")];
  const noteEl = root.querySelector("#gc-note span");
  const NOTES = {
    1: "L1 : seuls tes propres projets s'exécutent.",
    2: "L2 : projets open-source — ton offre est diffusée au réseau.",
    3: "L3 : uniquement une liste blanche que tu choisis.",
    4: "L4 : tout le public (double confirmation) — diffusée au réseau.",
  };
  const setLevel = (l) => {
    steps.forEach((s) => s.classList.toggle("step-primary", parseInt(s.dataset.lvl, 10) <= l));
    noteEl.textContent = NOTES[l];
    if (!REDUCE) {
      const lit = steps.filter((s) => parseInt(s.dataset.lvl, 10) <= l);
      if (lit.length) animate(lit, { opacity: [0.5, 1], duration: 300, delay: stagger(60), ease: "outQuad" });
    }
  };
  range.addEventListener("input", () => setLevel(parseInt(range.value, 10)));
  const watts = root.querySelector("#gc-watts");
  const vram = root.querySelector("#gc-vram");
  const hours = root.querySelector("#gc-hours");
  const setGauge = (el, val, over) => {
    el.style.setProperty("--value", Math.round(val));
    el.textContent = Math.round(val);
    el.className = "radial-progress " + (over ? "text-error" : "text-primary");
  };
  const animGauge = (el, to, over) => {
    if (REDUCE) return void setGauge(el, to, over);
    const o = { v: 0 };
    animate(o, { v: to, duration: 1200, ease: "outExpo", onUpdate: () => setGauge(el, o.v, over) });
  };
  const rejectBox = root.querySelector("#gc-reject");
  const rejectEl = rejectBox.querySelector("span");
  const TASKS = {
    ok: { w: 55, v: 62, h: 40, rej: null },
    watts: { w: 100, v: 62, h: 40, rej: "cap_watts", el: () => watts },
    hours: { w: 55, v: 62, h: 100, rej: "cap_hours_today", el: () => hours },
  };
  const runTask = (key) => {
    const t = TASKS[key];
    animGauge(watts, t.w, t.rej === "cap_watts");
    animGauge(vram, t.v, false);
    animGauge(hours, t.h, t.rej === "cap_hours_today");
    if (t.rej) {
      if (!REDUCE && t.el) animate(t.el(), { scale: [1, 1.06, 1], duration: 500, ease: "inOutSine" });
      rejectEl.innerHTML = `Refusée — <code>${t.rej}</code> dépassé.`;
      rejectBox.className = "alert alert-error text-xs py-2 mt-2";
    } else {
      rejectEl.textContent = "Caps respectés — la tâche passerait.";
      rejectBox.className = "alert alert-success text-xs py-2 mt-2";
    }
  };
  root.querySelectorAll("[data-gc-task]").forEach((b) => b.addEventListener("click", () => runTask(b.dataset.gcTask)));
  const model = root.querySelector("#gc-model");
  const reservoir = root.querySelector("#gc-reservoir");
  const fitEl = root.querySelector("#gc-fit");
  const GB = { 7: 4.4, 14: 8.5, 32: 20, 70: 42.5 };
  const BUDGET = 16;
  const renderFit = () => {
    const gb = GB[model.value] || 0;
    const over = gb > BUDGET;
    reservoir.innerHTML = "";
    const fill = document.createElement("span");
    fill.className = "gc-fill" + (over ? " is-over" : "");
    fill.style.width = Math.min(100, (gb / BUDGET) * 100) + "%";
    reservoir.appendChild(fill);
    if (!REDUCE) animate(fill, { scaleX: [0, 1], duration: 700, ease: "outExpo" });
    fitEl.innerHTML = over
      ? `<span class="text-error">${gb} Go &gt; 16 Go — ne tient pas</span> <span class="badge badge-xs badge-warning">sharding S77</span>`
      : `<span class="text-success">${gb} Go — tient sur un GPU</span>`;
  };
  model.addEventListener("change", renderFit);
  const io = new IntersectionObserver((es) => es.forEach((e) => {
    if (!e.isIntersecting) return;
    io.unobserve(e.target);
    setLevel(1);
    animGauge(watts, 55, false);
    animGauge(vram, 62, false);
    animGauge(hours, 40, false);
    renderFit();
  }), { threshold: 0.25 });
  io.observe(root);
})();

// ═══════════ Babel — SVG + texte animés ═══════════════════════════════════

// B1) Traduction en direct : splitText (entrée) + scrambleText (source→cible).
(() => {
  const src = document.getElementById("babel-source");
  const tgt = document.getElementById("babel-target-line");
  const langs = document.getElementById("babel-langs");
  const target = document.getElementById("babel-target");
  const replay = document.getElementById("babel-replay");
  if (!src || !tgt) return;
  const PAIRS = {
    "fr-en": { srcText: "Bonjour le monde", srcLang: "fr", tgt: { literal: "Hello world", idiom: "Hey there, world" }, tgtLang: "en" },
    "fr-sw": { srcText: "Bonjour le monde", srcLang: "fr", tgt: { literal: "Habari dunia", idiom: "Habari za dunia" }, tgtLang: "sw" },
    "fr-zu": { srcText: "Bonjour le monde", srcLang: "fr", tgt: { literal: "Sawubona mhlaba", idiom: "Sanibonani mhlaba" }, tgtLang: "zu" },
    "fr-ar": { srcText: "Bonjour le monde", srcLang: "fr", tgt: { literal: "مرحبا بالعالم", idiom: "أهلا بالعالم" }, tgtLang: "ar" },
  };
  let pairKey = "fr-en";
  const run = () => {
    const p = PAIRS[pairKey];
    const targetText = p.tgt[target.value] || p.tgt.literal;
    src.lang = p.srcLang;
    src.textContent = p.srcText; // réécrire AVANT re-split (sinon spans imbriqués)
    tgt.lang = p.tgtLang;
    if (REDUCE) {
      utils.set(src, { opacity: 1 });
      tgt.textContent = targetText;
      utils.set(tgt, { opacity: 1 });
      return;
    }
    const { chars } = splitText(src, { chars: true, accessible: true });
    utils.set(chars, { opacity: 0 });
    animate(chars, { y: [14, 0], opacity: [0, 1], rotateX: [-40, 0], duration: 520, delay: stagger(24), ease: "outExpo" });
    tgt.textContent = targetText; // poser la cible PUIS révéler son contenu courant
    utils.set(tgt, { opacity: 1 });
    animate(tgt, { innerHTML: scrambleText({ chars: "braille" }), duration: 1100, delay: 280, ease: "outQuad" });
  };
  langs.addEventListener("click", (e) => {
    const btn = e.target.closest("[data-pair]");
    if (!btn) return;
    langs.querySelectorAll(".btn").forEach((b) => b.classList.remove("btn-active"));
    btn.classList.add("btn-active");
    pairKey = btn.dataset.pair;
    run();
  });
  target.addEventListener("change", run);
  replay.addEventListener("click", run);
  run();
})();

// B2) Constellation de langues : arcs createDrawable + paquet createMotionPath.
(() => {
  const card = document.getElementById("babel-card");
  if (!card || !svg) return;
  const svgEl = document.getElementById("babel-svg");
  const packet = document.getElementById("babel-packet");
  const status = document.getElementById("babel-status");
  const arcs = [...svgEl.querySelectorAll(".babel-arc")];
  const targets = [...svgEl.querySelectorAll(".lang-target")];
  const drawables = arcs.map((p) => svg.createDrawable(p)[0]);
  const reset = () => {
    drawables.forEach((d) => utils.set(d, { draw: "0 0" }));
    utils.set(packet, { opacity: 0 });
    targets.forEach((g) => g.classList.remove("lang-arrived"));
  };
  if (REDUCE) {
    drawables.forEach((d) => utils.set(d, { draw: "0 1" }));
    targets.forEach((g) => g.classList.add("lang-arrived"));
    utils.set(packet, { opacity: 0 });
    status.textContent = "Traduit vers 6 langues — provenance vérifiée";
    return;
  }
  reset();
  const translateTo = (i) =>
    new Promise((resolve) => {
      animate(drawables[i], { draw: ["0 0", "0 1"], duration: 420, ease: "inOutSine" });
      utils.set(packet, { opacity: 1 });
      animate(packet, {
        ...svg.createMotionPath("#" + arcs[i].id),
        duration: 720,
        ease: "inOutQuad",
        onComplete: () => { targets[i].classList.add("lang-arrived"); utils.set(packet, { opacity: 0 }); resolve(); },
      });
    });
  let running = false;
  const run = async () => {
    if (running) return;
    running = true;
    reset();
    for (let i = 0; i < arcs.length; i++) {
      status.textContent = `Traduction ${i + 1}/${arcs.length}…`;
      await translateTo(i);
    }
    status.textContent = "Traduit vers 6 langues — provenance vérifiée";
    running = false;
  };
  card.querySelectorAll("[data-babel-run]").forEach((b) => b.addEventListener("click", run));
})();

// B3) Un glyphe, toutes les écritures : svg.morphTo sur 'd' (même type d'élément).
(() => {
  const glyph = document.getElementById("babel-glyph");
  if (!glyph || !svg) return;
  const STEPS = [
    { id: "#g-latin", char: "A", script: "latine", lang: "anglais → français" },
    { id: "#g-hira", char: "あ", script: "hiragana", lang: "japonais → français" },
    { id: "#g-arab", char: "ع", script: "arabe", lang: "arabe → swahili" },
    { id: "#g-cyr", char: "Я", script: "cyrillique", lang: "russe → yoruba" },
  ];
  const charEl = document.getElementById("babel-char");
  const scriptEl = document.getElementById("babel-script");
  const langEl = document.getElementById("babel-lang");
  const consEl = document.getElementById("babel-consensus");
  const sigEl = document.getElementById("babel-sig");
  const scramble = (el, txt) => {
    if (REDUCE) return void (el.textContent = txt);
    const pool = "abcdefghijklmnopqrstuvwxyzàçéèⁿ→ ";
    const obj = { p: 0 };
    animate(obj, { p: 1, duration: 700, ease: "outQuad", onUpdate: () => {
      const shown = Math.floor(obj.p * txt.length);
      let out = txt.slice(0, shown);
      for (let i = shown; i < txt.length; i++) out += txt[i] === " " ? " " : pool[Math.floor(Math.random() * pool.length)];
      el.textContent = out;
    } });
  };
  let i = 0;
  const applyText = (s) => { charEl.textContent = s.char; scriptEl.textContent = s.script; scramble(langEl, s.lang); };
  if (REDUCE) { applyText(STEPS[0]); consEl.classList.add("step-primary"); return; }
  const goTo = (next) => {
    const s = STEPS[next];
    try {
      animate(glyph, {
        d: svg.morphTo(s.id, 0.35),
        duration: 900,
        ease: "inOutQuad",
        onBegin: () => { consEl.classList.remove("step-primary"); sigEl.classList.remove("badge-success"); sigEl.classList.add("badge-ghost"); },
        onComplete: () => { applyText(s); consEl.classList.add("step-primary"); sigEl.classList.remove("badge-ghost"); sigEl.classList.add("badge-success"); },
      });
    } catch (_) { applyText(s); } // morphTo lève sur cible invalide : la boucle survit
    i = next;
  };
  applyText(STEPS[0]);
  let timer = setInterval(() => goTo((i + 1) % STEPS.length), 2400);
  document.querySelectorAll("[data-babel-step]").forEach((b) =>
    b.addEventListener("click", () => { clearInterval(timer); goTo((i + 1) % STEPS.length); timer = setInterval(() => goTo((i + 1) % STEPS.length), 2400); }),
  );
})();

// B4) Provenance signée : createDrawable (tracé) + createMotionPath (jeton).
(() => {
  const root = document.getElementById("babel-prov");
  if (!root || !svg) return;
  const drawPath = document.getElementById("bp-draw");
  const track = document.getElementById("bp-track");
  const nodes = [...root.querySelectorAll(".bp-node")];
  const token = document.getElementById("bp-token");
  const roleBadge = document.getElementById("bp-role");
  const count = document.getElementById("bp-count");
  const ROLES = ["source_work", "source_manifest", "source_chunker", "translator_worker", "auto_validator", "human_corrector", "human_reviewer", "consensus_attestor", "publisher"];
  const N = nodes.length;
  // jeton en boucle le long du tracé de fond (indépendant du dessin)
  if (!REDUCE) animate(token, { ...svg.createMotionPath("#bp-track"), duration: 3600, loop: true, ease: "linear" });
  if (REDUCE) {
    const [drawable] = svg.createDrawable(drawPath);
    utils.set(drawable, { draw: "0 1" });
    nodes.forEach((n) => n.classList.add("is-signed"));
    roleBadge.textContent = "9 enregistrements signés";
    count.textContent = "9 / 9 signés";
    const end = track.getPointAtLength(track.getTotalLength());
    utils.set(token, { translateX: end.x, translateY: end.y });
    return;
  }
  utils.set(svg.createDrawable(drawPath)[0], { draw: "0 0" });
  const run = () => {
    nodes.forEach((n) => n.classList.remove("is-signed"));
    const [drawable] = svg.createDrawable(drawPath);
    const obj = { p: 0 };
    animate(obj, {
      p: 1,
      duration: 4200,
      ease: "linear",
      onUpdate: () => {
        utils.set(drawable, { draw: "0 " + obj.p });
        const lit = Math.min(N - 1, Math.floor(obj.p * (N - 1) + 0.0001));
        for (let k = 0; k <= lit; k++) nodes[k].classList.add("is-signed");
        roleBadge.textContent = ROLES[lit];
        count.textContent = lit + 1 + " / 9 signés";
      },
      onComplete: () => { roleBadge.textContent = "9 enregistrements signés"; },
    });
  };
  root.querySelectorAll("[data-babel-prov-run]").forEach((b) => b.addEventListener("click", run));
  count.textContent = "0 / 9 signés";
})();

// B5) GPU multi-nœud : createTimeline (handoffs) + jeton createMotionPath synchronisé.
(() => {
  const root = document.getElementById("sp-pipeline");
  if (!root || !svg) return;
  const token = root.querySelector("#sp-token");
  const cards = [...root.querySelectorAll(".sp-card")];
  const ttftEl = root.querySelector("#sp-ttft");
  const tpsEl = root.querySelector("#sp-tps");
  const rttEl = root.querySelector("#sp-rtt");
  const verifEl = root.querySelector("#sp-verif");
  const statusEl = root.querySelector("#sp-status");
  const toggle = root.querySelector("#sp-toggle");
  const TTFT = 740, TPS = 2.4, RTT = 38, T = 520;
  const light = (idx, on) => cards[idx] && cards[idx].classList.toggle("is-active", on);
  // jeton : créé en pause, piloté par la timeline (ou figé en reduced-motion)
  const motion = animate(token, { ...svg.createMotionPath("#sp-path"), duration: T * (cards.length - 1), ease: "linear", autoplay: false });
  if (REDUCE) {
    cards.forEach((c) => c.classList.add("is-active"));
    motion.seek(motion.duration);
    ttftEl.textContent = `${TTFT} ms`; tpsEl.textContent = `${TPS} tok/s`; rttEl.textContent = `${RTT} ms`;
    verifEl.textContent = "N0 → N2 ✓"; statusEl.textContent = "vérifié"; statusEl.className = "badge badge-sm badge-success";
    toggle.disabled = true;
    return;
  }
  const tl = createTimeline({
    loop: true, loopDelay: 600, defaults: { duration: 420, ease: "outQuad" }, autoplay: false,
    onLoop: () => {
      cards.forEach((c) => c.classList.remove("is-active"));
      verifEl.textContent = "N0 → N2"; verifEl.classList.remove("text-success");
      statusEl.textContent = "en boucle"; statusEl.className = "badge badge-sm";
    },
  });
  tl.sync(motion, 0);
  cards.forEach((_, idx) =>
    tl.add(cards[idx], {
      opacity: [0.5, 1], scale: [0.97, 1],
      onBegin: () => {
        light(idx, true);
        if (idx === 1) ttftEl.textContent = `${TTFT} ms`;
        if (idx === 2) rttEl.textContent = `${RTT} ms`;
        if (idx === 3) tpsEl.textContent = `${TPS} tok/s`;
        if (idx === 4) {
          verifEl.textContent = "N0 → N2 ✓"; verifEl.classList.add("text-success");
          statusEl.textContent = "RunProof signé · N2"; statusEl.className = "badge badge-sm badge-success";
        }
      },
    }, idx * T),
  );
  let playing = true;
  toggle.addEventListener("click", () => {
    playing = !playing;
    if (playing) { tl.play(); toggle.textContent = "Pause"; statusEl.textContent = "en boucle"; }
    else { tl.pause(); toggle.textContent = "Reprendre"; statusEl.textContent = "en pause"; }
    toggle.setAttribute("aria-pressed", String(playing));
  });
  const io = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { io.unobserve(e.target); tl.play(); } }), { threshold: 0.25 });
  io.observe(root);
})();

// ═══════════ Combos profonds (plusieurs modules anime ensemble) ════════════

// A. Intro cinétique : 1 timeline orchestre splitText+stagger, createDrawable,
//    scrambleText et un ressort createSpring — tout verrouillé sur le même axe.
(() => {
  const root = document.getElementById("ci-card");
  if (!root || !svg) return;
  const title = root.querySelector("#ci-title");
  const sub = root.querySelector("#ci-sub");
  const seal = root.querySelector("#ci-seal");
  const SUB = "Traduction vérifiable, pair à pair";
  const run = () => {
    title.textContent = "Babel"; // réécrire AVANT re-split
    sub.textContent = SUB;
    if (REDUCE) {
      utils.set(svg.createDrawable("#ci-underline-path")[0], { draw: "0 1" });
      utils.set([title, seal], { opacity: 1, translateY: 0, scale: 1, rotateX: 0 });
      return;
    }
    const { chars } = splitText(title, { chars: true, accessible: true });
    utils.set(chars, { opacity: 0 });
    const [underline] = svg.createDrawable("#ci-underline-path");
    utils.set(underline, { draw: "0 0" });
    utils.set(seal, { opacity: 0, translateY: 18, scale: 0.9 });
    createTimeline({ defaults: { ease: "out(3)" } })
      .add(chars, { opacity: [0, 1], translateY: [28, 0], rotateX: [-90, 0], duration: 520, delay: stagger(48, { from: "center" }) }, 0)
      .add(underline, { draw: ["0 0", "0 1"], duration: 700, ease: "inOut(2)" }, 0)
      .add(sub, { innerHTML: scrambleText({ chars: "braille" }), duration: 900, ease: "out(2)" }, "<+=80")
      .add(seal, { opacity: [0, 1], translateY: [18, 0], scale: [0.9, 1], duration: 800, ease: spring({ stiffness: 120, damping: 9 }) }, "-=160");
  };
  root.querySelector("[data-ci-replay]")?.addEventListener("click", run);
  const io = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { io.unobserve(e.target); run(); } }), { threshold: 0.3 });
  io.observe(root);
})();

// B. Panier à kudos : createDraggable+spring + createAnimatable (aiguille curseur)
//    + compteur animate/createSpring + stagger d'entrée.
(() => {
  const root = document.getElementById("kb-card");
  if (!root) return;
  const zone = root.querySelector("#kb-zone");
  const basket = root.querySelector("#kb-basket");
  const gauge = root.querySelector("#kb-gauge");
  const scoreEl = root.querySelector("#kb-score");
  const chips = [...root.querySelectorAll(".kb-chip")];
  let score = 0;
  const setScore = (v) => { gauge.style.setProperty("--value", Math.min(100, v)); gauge.textContent = v; scoreEl.textContent = `${v} kudos`; };
  setScore(0);
  if (!REDUCE) animate(chips, { opacity: [0, 1], scale: [0.6, 1], duration: 420, delay: stagger(70), ease: "outBack" });
  const addKudos = (amt) => {
    const from = score; score += amt;
    if (REDUCE) return void setScore(score);
    const o = { v: from };
    animate(o, { v: score, duration: 900, ease: spring({ stiffness: 90, damping: 11 }), onUpdate: () => setScore(Math.round(o.v)) });
    animate(gauge, { scale: [1, 1.1, 1], duration: 420, ease: "outQuad" });
  };
  // createAnimatable : aiguille qui suit le curseur dans la carte (temps réel).
  if (!REDUCE && typeof createAnimatable === "function") {
    const needle = createAnimatable("#kb-needle", { rotate: 260, ease: "out(3)" });
    root.addEventListener("pointermove", (e) => {
      const r = root.getBoundingClientRect();
      needle.rotate(((e.clientX - r.left) / r.width - 0.5) * 120);
    });
  }
  const commit = (chip) => {
    if (chip.dataset.done) return;
    chip.dataset.done = "1";
    addKudos(parseInt(chip.dataset.amt, 10) || 1);
    if (REDUCE) { chip.style.visibility = "hidden"; return; }
    animate(chip, { opacity: [1, 0], scale: [1, 0.4], duration: 300, ease: "outQuad", onComplete: () => (chip.style.visibility = "hidden") });
  };
  if (!REDUCE && typeof createDraggable === "function") {
    chips.forEach((chip) =>
      createDraggable(chip, {
        container: zone,
        releaseEase: spring({ stiffness: 160, damping: 14 }),
        onRelease: () => {
          const c = chip.getBoundingClientRect(), b = basket.getBoundingClientRect();
          if (c.left < b.right && c.right > b.left && c.top < b.bottom && c.bottom > b.top) commit(chip);
        },
      }),
    );
  }
  chips.forEach((chip) => chip.addEventListener("dblclick", () => commit(chip))); // fallback accessible
})();

// ───────────────────────── Toasts (@starting-style à l'insertion) ──────────
(() => {
  const host = document.getElementById("toasts");
  if (!host) return;
  let n = 0;
  const show = (msg) => {
    const t = document.createElement("div");
    t.className = "alert alert-success sbfb-toast shadow-lg";
    t.setAttribute("role", "status");
    const span = document.createElement("span");
    span.textContent = `${++n}. ${msg}`;
    t.appendChild(span);
    host.appendChild(t); // entrée animée par @starting-style
    setTimeout(() => {
      t.classList.add("is-leaving");
      setTimeout(() => t.remove(), 320);
    }, 2800);
  };
  document.querySelectorAll("[data-toast]").forEach((b) =>
    b.addEventListener("click", () => show("Toast animé par @starting-style.")),
  );
})();

// ───────────────────────── Sismographe : engine.speed piloté par le jerk ───
// Le temps (engine.speed, GLOBAL) plonge au ralenti quand le jerk — 3e dérivée
// de la position du stylet — dépasse un seuil. Deux horloges, rôles séparés :
//  • Horloge-MONDE = createTimer (tickable → ralentit AVEC engine.speed) : calcule
//    le signal, déplace l'aiguille, fait défiler le papier (au rythme de l'anime-temps).
//  • DÉTECTEUR = requestAnimationFrame NATIF (temps réel, JAMAIS ralenti par
//    engine.speed) : dérive vitesse→accél→jerk par cascade de damp et pilote
//    engine.speed. Point cardinal du concept : engine.speed ne touche pas le rAF
//    natif, donc l'horloge de détection reste fidèle au temps réel.
// Auto-limitant : pendant le gel le signal avance lentement → le jerk mesuré
// retombe → le temps repart. engine.speed étant GLOBAL, on gate par
// IntersectionObserver et on rétablit speed=1 dès que la section sort de vue.
(() => {
  const A = window.anime;
  if (!A) return;
  const { utils, createTimer, engine } = A;
  const stage = document.getElementById("seis-stage");
  if (!stage || !engine || typeof createTimer !== "function") return;

  const W = 600, H = 200, N = 170;
  const MID = H / 2, AMP = 58;
  const FREEZE_MIN = 0.06;             // engine.speed au gel maximal
  const LOOP_MS = 24000;              // période de l'horloge-monde
  const LOOP_S = LOOP_MS / 1000;
  const SAMPLE_MS = LOOP_MS / N;      // un échantillon de papier par tranche d'anime-temps
  const TWO_PI = Math.PI * 2;
  const w1 = TWO_PI / LOOP_S;         // base de fréquence commensurable avec la boucle

  const trace = document.getElementById("seis-trace");
  const needle = document.getElementById("seis-needle");
  const elState = document.getElementById("seis-state");
  const elJerk = document.getElementById("seis-jerk");
  const elPct = document.getElementById("seis-freeze-pct");
  const elQuakes = document.getElementById("seis-quakes");

  // Graduations : statiques, écrites en markup SVG (le namespace createElementNS
  // serait une URL http interdite par check:csp dans les fichiers authored).

  // RNG seedé déterministe (mulberry32) → mêmes secousses à chaque chargement
  // d'iframe (preview reproductible, lever « génératif reproductible »).
  const mulberry32 = (a) => () => {
    a |= 0; a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  // Secousses pré-générées dans [1s, LOOP_S-1.4s] (jamais à cheval sur le bouclage).
  const quakes = [];
  {
    const rng = mulberry32(0x5b1f);
    let t = 1.0;
    while (t < LOOP_S - 1.4) {
      quakes.push({
        t0: t,
        amp: 0.55 + rng() * 0.5,
        freq: 26 + rng() * 22,        // oscillation rapide = forte 3e dérivée
        decay: 7 + rng() * 5,
        sign: rng() < 0.5 ? -1 : 1,
      });
      t += 2.0 + rng() * 2.3;
    }
  }
  const TOTAL_QUAKES = quakes.length;

  // Signal normalisé ~[-1.6,1.6] : dérive calme PÉRIODIQUE (continue au bouclage)
  // + paquets d'ondes sismiques brusques (forte courbure → fort jerk).
  const signal = (tSec) => {
    let s = 0.42 * Math.sin(tSec * w1 * 2) + 0.16 * Math.sin(tSec * w1 * 5 + 0.6);
    for (let i = 0; i < quakes.length; i++) {
      const q = quakes[i];
      const u = tSec - q.t0;
      if (u >= 0 && u < 1.2) s += q.sign * q.amp * Math.exp(-u * q.decay) * Math.sin(u * q.freq);
    }
    return s;
  };

  const yOf = (sig) => utils.clamp(MID - sig * AMP, 6, H - 6);
  const xOf = (i) => (i / (N - 1)) * W;

  // Tampon d'historique du tracé (le « papier » qui défile).
  const buf = new Array(N);
  const renderTrace = () => {
    let pts = "";
    for (let i = 0; i < N; i++) pts += xOf(i).toFixed(1) + "," + yOf(buf[i]).toFixed(2) + " ";
    trace.setAttribute("points", pts.trim());
  };
  // Pré-remplissage calme pour un état initial lisible.
  for (let i = 0; i < N; i++) buf[i] = 0.42 * Math.sin((i / N) * LOOP_S * w1 * 2);
  renderTrace();
  needle.setAttribute("cy", yOf(buf[N - 1]).toFixed(2));

  // État final lisible si prefers-reduced-motion : tracé calme + une secousse,
  // aiguille au repos, 0 gel, aucune boucle, engine.speed intouché.
  if (REDUCE) {
    for (let i = 0; i < N; i++) buf[i] = signal((i / N) * LOOP_S);
    renderTrace();
    needle.setAttribute("cy", yOf(buf[N - 1]).toFixed(2));
    elQuakes.textContent = String(TOTAL_QUAKES);
    return;
  }

  // ── Horloge-MONDE : createTimer (ralentit avec engine.speed) ──
  let currentSig = 0;
  let acc = 0;
  const world = createTimer({
    duration: LOOP_MS,
    loop: true,
    autoplay: false,
    onLoop: () => { acc = 0; },        // évite un faux jerk au bouclage
    onUpdate: (self) => {
      currentSig = signal(self.currentTime / 1000);
      needle.setAttribute("cy", yOf(currentSig).toFixed(2));
      acc += self.deltaTime;           // avance le papier au rythme de l'ANIME-temps
      let pushed = false;
      while (acc >= SAMPLE_MS) { buf.shift(); buf.push(currentSig); acc -= SAMPLE_MS; pushed = true; }
      if (pushed) renderTrace();
    },
  });

  // ── DÉTECTEUR de jerk : rAF NATIF (temps réel, jamais ralenti) ──
  let rafId = 0, running = false, prevNow = 0;
  let py = 0, vy = 0, ay = 0, jy = 0, pv = 0, pa = 0, freezeNow = 0, env = 0;
  let quakeCount = 0, lastFreeze = 0, textTick = 0;
  window.__seis = { freeze: 0, jerk: 0, speed: 1, quakes: 0 };

  const tick = (now) => {
    if (!running) return;
    let dt = prevNow ? now - prevNow : 16;
    dt = utils.clamp(dt, 1, 64);       // borne dt (onglet caché / hoquet → pas de faux jerk)
    prevNow = now;

    const y = currentSig;              // position réelle du stylet, lue en TEMPS RÉEL
    // Cascade de différences finies lissées par damp (frame-rate-indépendant).
    vy = utils.damp(vy, y - py, dt, 0.4); py = y;
    ay = utils.damp(ay, vy - pv, dt, 0.35); pv = vy;
    jy = utils.damp(jy, ay - pa, dt, 0.3); pa = ay;
    const jerkMag = Math.abs(jy);

    // Enveloppe attack-instant / release-lent sur le jerk : un à-coup déclenche
    // un PALIER de gel qui tient ~0.8s puis relâche. Sans elle, l'auto-limitation
    // (le signal ralentit → le jerk mesuré retombe) collapse le gel en 2 frames
    // → flicker au lieu d'un vrai ralenti perceptible.
    const trigger = utils.clamp(utils.mapRange(jerkMag, 0.005, 0.04, 0, 1), 0, 1);
    env = Math.max(trigger, env - dt * 0.0012);
    freezeNow = utils.damp(freezeNow, env, dt, 0.22);
    engine.speed = utils.lerp(1, FREEZE_MIN, freezeNow);
    stage.style.setProperty("--seis-freeze", freezeNow.toFixed(3));

    if (lastFreeze < 0.5 && freezeNow >= 0.5) quakeCount++;   // front montant = secousse ressentie
    lastFreeze = freezeNow;

    const dbg = window.__seis;
    dbg.freeze = freezeNow; dbg.jerk = jerkMag; dbg.speed = engine.speed; dbg.quakes = quakeCount;

    if ((textTick++ & 7) === 0) {
      elState.textContent = "vitesse ×" + engine.speed.toFixed(2);
      elJerk.textContent = jerkMag.toFixed(3);
      elPct.textContent = Math.round(freezeNow * 100) + " %";
      elQuakes.textContent = String(quakeCount);
    }
    rafId = requestAnimationFrame(tick);
  };

  const start = () => {
    if (running) return;
    running = true; prevNow = 0;
    py = currentSig; vy = ay = jy = pv = pa = 0; freezeNow = 0; env = 0; lastFreeze = 0;
    world.play();
    rafId = requestAnimationFrame(tick);
  };
  const stop = () => {
    if (!running) return;
    running = false;
    if (rafId) cancelAnimationFrame(rafId);
    world.pause();
    engine.speed = 1;                  // rétablit le temps global dès la sortie de vue
    freezeNow = 0;
    stage.style.setProperty("--seis-freeze", "0");
    if (window.__seis) { window.__seis.freeze = 0; window.__seis.speed = 1; }
    elState.textContent = "vitesse ×1.00";
  };

  // engine.speed est GLOBAL : on ne l'anime QUE pendant que la section est visible.
  const io = new IntersectionObserver(
    (es) => es.forEach((e) => (e.isIntersecting ? start() : stop())),
    { threshold: 0.25 },
  );
  io.observe(stage);
})();

// ───────────────────────── Train d'engrenages : pipeline de shards ─────────
// Re-thématisation SBFB du concept #4. Chaque engrenage = un shard. Une impulsion
// sur la roue motrice (= un batch d'activations qui arrive) propage une ONDE de
// vitesse angulaire de roue en roue, avec retard et amortissement (couplage damp
// amont→aval à coefficient DÉCROISSANT = RTT relais + buffering KV). On voit
// l'activation traverser les couches. La tension |Δvitesse| entre roues = le
// goulot de débit (le shard le plus lent borne les tok/s). Backlash = jeu
// mécanique qui tremble dans les à-coups. État JS pur + utils + un createTimer ;
// 0 réseau, données simulées. Rotation posée via attribut SVG transform (robuste,
// pas de quirk transform-box) ; porteuse (vitesse propagée) + modulation (backlash
// lié à la tension) sommées analytiquement.
(() => {
  const A = window.anime;
  if (!A) return;
  const { utils, createTimer } = A;
  const stage = document.getElementById("gears-stage");
  if (!stage || typeof createTimer !== "function") return;

  const rots = Array.from(stage.querySelectorAll(".gear-rot"));
  const gears = Array.from(stage.querySelectorAll(".gear"));
  const proofGear = stage.querySelector(".gear.is-proof");
  const bneck = stage.querySelector(".gear-bottleneck");
  const elTps = document.getElementById("ge-tps");
  const elStage = document.getElementById("ge-stage");
  const elProofs = document.getElementById("ge-proofs");
  const elGoulot = document.getElementById("ge-goulot");
  const N = rots.length;
  if (!N) return;

  const GX = [55, 100, 145, 190, 235, 280];
  const GY = [110, 84, 110, 84, 110, 84];
  const NAMES = ["Dispatch", "Shard 1", "Shard 2", "Shard 3", "Shard 4", "RunProof"];
  const dir = (i) => (i % 2 ? -1 : 1);

  const VBASE = 1.9, IMP = 4.2, ROT = 0.026, BL = 2.6, STRAIN_MAX = 2.6;
  // Couplage décroissant = retard de transmission croissant vers l'aval.
  const K = [0.3, 0.3, 0.26, 0.22, 0.19, 0.16];
  // L'onde de tension s'atténue vers l'aval (chaque damp lisse les écarts) ; ce gain
  // croissant la ré-égalise pour qu'on VOIE l'activation toucher chaque shard.
  const GAIN = [0.7, 1.3, 2.3, 3.8, 6.0, 9.0];

  const angle = new Array(N).fill(0);
  const vel = new Array(N).fill(VBASE);
  const strain = new Array(N).fill(0);
  const heatVal = new Array(N).fill(0);
  const setRot = (i, deg) => rots[i].setAttribute("transform", `rotate(${deg.toFixed(2)} 0 0)`);

  // reduced-motion : engrenages au repos, alignés, 0 chaleur, 0 goulot.
  if (REDUCE) {
    for (let i = 0; i < N; i++) { setRot(i, 0); gears[i].style.setProperty("--heat", "0"); }
    if (bneck) bneck.style.opacity = "0";
    if (elTps) elTps.textContent = "—";
    if (elStage) elStage.textContent = "au repos";
    if (elGoulot) elGoulot.textContent = "—";
    return;
  }

  let proofs = 0, lastTail = 0, textTick = 0, dispVel = VBASE;
  window.__gears = { tps: 0, stage: 0, proofs: 0, bottleneck: 0, maxStrain: 0 };

  // Impulsion auto-pilotée : un batch d'activations arrive ~toutes les 2.6 s.
  const impulse = (tMs) => {
    const ph = (tMs % 3800) / 3800;
    return ph < 0.18 ? ((1 - Math.cos((ph / 0.18) * Math.PI)) / 2) * IMP : 0;
  };

  const timer = createTimer({
    duration: 60000, loop: true, autoplay: false,
    onUpdate: (self) => {
      const dt = utils.clamp(self.deltaTime, 1, 64);
      const t = self.currentTime;
      const target0 = VBASE + impulse(t);
      for (let i = 0; i < N; i++) {
        const tgt = i === 0 ? target0 : vel[i - 1];     // cible aval = vitesse amont
        vel[i] = utils.damp(vel[i], tgt, dt, K[i]);       // retard croissant (K décroissant)
        strain[i] = Math.abs(tgt - vel[i]);               // tension = en train de rattraper = calcule
        // backlash : jeu mécanique, tremble plus quand la tension monte
        const bl = BL * Math.sin(t * 0.014 + i * 1.7) * (0.18 + utils.clamp(strain[i] / STRAIN_MAX, 0, 1));
        angle[i] += vel[i] * dt * ROT;                    // accumulateur non-wrappé
        setRot(i, utils.wrap(dir(i) * angle[i], 0, 360) + bl); // wrap AU RENDU + modulation
        // Chaleur → la roue vire au JAUNE : on pose --heat, le color-mix CSS fait le reste.
        heatVal[i] = utils.clamp(utils.mapRange(strain[i] * GAIN[i], 0.04, 1.0, 0, 0.9), 0, 0.9);
        gears[i].style.setProperty("--heat", heatVal[i].toFixed(3));
      }
      // Goulot = roue la plus lente (borne le débit). Débit ∝ min vel.
      let minV = Infinity, minI = 1, maxS = 0, maxSI = 0;
      for (let i = 1; i < N; i++) if (vel[i] < minV) { minV = vel[i]; minI = i; }
      for (let i = 0; i < N; i++) if (strain[i] > maxS) { maxS = strain[i]; maxSI = i; }
      dispVel = utils.damp(dispVel, minV, dt, 0.08);
      if (bneck) {
        bneck.setAttribute("transform", `translate(${GX[minI]} ${GY[minI]})`);
        bneck.style.opacity = utils.clamp(utils.mapRange(VBASE - minV, 0.05, 1.2, 0, 0.9), 0, 0.9).toFixed(3);
      }
      // RunProof : front montant de la chaleur sur la dernière roue = onde arrivée.
      const tail = heatVal[N - 1];
      if (lastTail < 0.3 && tail >= 0.3) {
        proofs++;
        if (proofGear) { proofGear.classList.add("is-firing"); setTimeout(() => proofGear.classList.remove("is-firing"), 260); }
      }
      lastTail = tail;

      const g = window.__gears;
      g.tps = dispVel; g.stage = maxSI; g.proofs = proofs; g.bottleneck = minI; g.maxStrain = maxS;
      if ((textTick++ & 7) === 0) {
        if (elTps) elTps.textContent = Math.round(dispVel * 9) + " tok/s";
        if (elStage) elStage.textContent = NAMES[maxSI];
        if (elProofs) elProofs.textContent = String(proofs);
        if (elGoulot) elGoulot.textContent = NAMES[minI];
      }
    },
  });

  const io = new IntersectionObserver(
    (es) => es.forEach((e) => (e.isIntersecting ? timer.play() : timer.pause())),
    { threshold: 0.2 },
  );
  io.observe(stage);
})();
