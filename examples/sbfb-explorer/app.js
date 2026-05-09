// SPDX-License-Identifier: AGPL-3.0-or-later

(function () {
  "use strict";

  var REPO_BASE = "https://github.com/SBFB50/SBFB/tree/master/";

  // F2: wire source links to point at the repo
  var srcLinks = document.querySelectorAll("a.src[data-path]");
  for (var i = 0; i < srcLinks.length; i++) {
    srcLinks[i].href = REPO_BASE + srcLinks[i].getAttribute("data-path");
    srcLinks[i].target = "_blank";
    srcLinks[i].rel = "noopener";
  }

  // Status panel toggle
  var panel = document.getElementById("status-panel");
  var toggle = document.getElementById("status-toggle");
  toggle.addEventListener("click", function () {
    var isHidden = panel.hasAttribute("hidden");
    if (isHidden) {
      panel.removeAttribute("hidden");
    } else {
      panel.setAttribute("hidden", "");
    }
  });

  // F3: live status via bridge
  var dot = document.getElementById("status-dot");
  var nodeIdEl = document.getElementById("node-id");
  var pubkeyEl = document.getElementById("pubkey");
  var versionEl = document.getElementById("version");
  var uptimeEl = document.getElementById("uptime");
  var peersEl = document.getElementById("peers");
  var appListEl = document.getElementById("app-list");
  var footerEl = document.getElementById("status-footer");

  var bridge = null;
  try {
    bridge = new SBFBBridge({ timeout: 5000 });
  } catch (e) {
    setOffline("Bridge non disponible");
    return;
  }

  function setOnline() {
    dot.className = "status-dot online";
  }

  function setOffline(reason) {
    dot.className = "status-dot offline";
    footerEl.textContent = reason || "Daemon non connecté";
  }

  function formatUptime(seconds) {
    if (!seconds && seconds !== 0) return "—";
    var h = Math.floor(seconds / 3600);
    var m = Math.floor((seconds % 3600) / 60);
    var s = Math.floor(seconds % 60);
    if (h > 0) return h + "h " + m + "m";
    if (m > 0) return m + "m " + s + "s";
    return s + "s";
  }

  function truncateId(id) {
    if (!id) return "—";
    if (id.length <= 16) return id;
    return id.slice(0, 8) + "…" + id.slice(-8);
  }

  function fetchStatus() {
    var statusOk = false;

    bridge
      .getNodeStatus()
      .then(function (data) {
        statusOk = true;
        setOnline();
        nodeIdEl.textContent = truncateId(data.node_id);
        nodeIdEl.title = data.node_id || "";
        versionEl.textContent = data.version || "—";
        uptimeEl.textContent = formatUptime(data.uptime_seconds);
        peersEl.textContent =
          data.peers !== undefined ? String(data.peers) : "—";
        footerEl.textContent =
          "Dernière mise à jour : " + new Date().toLocaleTimeString();
      })
      .catch(function () {
        if (!statusOk) setOffline("Daemon non connecté");
      });

    bridge
      .getIdentityPubkey()
      .then(function (data) {
        pubkeyEl.textContent = truncateId(data.pubkey);
        pubkeyEl.title = data.pubkey || "";
      })
      .catch(function () {
        pubkeyEl.textContent = "—";
      });

    bridge
      .getBrowseList()
      .then(function (data) {
        var entries = data.entries || [];
        appListEl.innerHTML = "";
        if (entries.length === 0) {
          var li = document.createElement("li");
          li.className = "placeholder";
          li.textContent = "Aucune app disponible";
          appListEl.appendChild(li);
          return;
        }
        for (var j = 0; j < entries.length; j++) {
          var item = document.createElement("li");
          item.textContent = entries[j].name || entries[j].project_name || "App sans nom";
          appListEl.appendChild(item);
        }
      })
      .catch(function () {
        appListEl.innerHTML = "";
        var li = document.createElement("li");
        li.className = "placeholder";
        li.textContent = "Non disponible";
        appListEl.appendChild(li);
      });
  }

  fetchStatus();
  setInterval(fetchStatus, 15000);
})();
