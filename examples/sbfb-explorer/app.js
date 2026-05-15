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

  // F4: verification & provenance demo
  var projectSelect = document.getElementById("verify-project");
  var verifyBtn = document.getElementById("verify-btn");
  var verifyResult = document.getElementById("verify-result");
  var verifyStatus = document.getElementById("verify-status");
  var verifyFields = document.getElementById("verify-fields");

  function populateProjects() {
    bridge
      .getBrowseList()
      .then(function (data) {
        var entries = data.entries || [];
        projectSelect.innerHTML = "";
        if (entries.length === 0) {
          var opt = document.createElement("option");
          opt.value = "";
          opt.textContent = "Aucun projet disponible";
          projectSelect.appendChild(opt);
          verifyBtn.disabled = true;
          return;
        }
        for (var k = 0; k < entries.length; k++) {
          var opt = document.createElement("option");
          opt.value = entries[k].project_id || entries[k].id || "";
          opt.setAttribute("data-hash", entries[k].provenance_hash || "");
          opt.textContent = entries[k].name || entries[k].project_name || "Projet sans nom";
          projectSelect.appendChild(opt);
        }
        verifyBtn.disabled = false;
      })
      .catch(function () {
        projectSelect.innerHTML = "";
        var opt = document.createElement("option");
        opt.value = "";
        opt.textContent = "Erreur de chargement";
        projectSelect.appendChild(opt);
        verifyBtn.disabled = true;
      });
  }

  populateProjects();

  verifyBtn.addEventListener("click", function () {
    var projectId = projectSelect.value;
    if (!projectId) return;

    var selectedOpt = projectSelect.options[projectSelect.selectedIndex];
    var announceHash = selectedOpt ? selectedOpt.getAttribute("data-hash") : "";

    verifyBtn.disabled = true;
    verifyResult.removeAttribute("hidden");
    verifyStatus.className = "verify-status loading";
    verifyStatus.textContent = "Vérification en cours…";
    verifyFields.innerHTML = "";

    bridge
      .verifyRelease(projectId)
      .then(function (data) {
        var hashMismatch =
          data.verified &&
          announceHash &&
          data.provenance_hash &&
          data.provenance_hash !== announceHash;

        if (data.verified && !hashMismatch) {
          verifyStatus.className = "verify-status verified";
          verifyStatus.textContent = "Provenance vérifiée";
        } else if (data.verified && hashMismatch) {
          verifyStatus.className = "verify-status unverified";
          verifyStatus.textContent =
            "Signature valide mais hash ne correspond pas à l'annonce réseau";
        } else {
          verifyStatus.className = "verify-status unverified";
          verifyStatus.textContent = data.error || "Provenance non vérifiée";
        }

        var record = data.record;
        if (record) {
          var fields = [
            ["Dépôt", record.repo_url || "—"],
            ["Commit", record.commit_sha || "—"],
            ["Artifact hash", record.artifact_hash || "—"],
            ["Signature", truncateId(record.signature)],
            ["Noeud", truncateId(record.node_id)],
            ["Date", record.timestamp || "—"],
            ["Schema", "v" + (record.schema_version || 1)]
          ];
          var html = "";
          for (var m = 0; m < fields.length; m++) {
            html +=
              "<dt>" + fields[m][0] + "</dt>" +
              '<dd title="' + escapeAttr(fields[m][1]) + '">' + escapeHtml(fields[m][1]) + "</dd>";
          }
          verifyFields.innerHTML = html;
        }

        verifyBtn.disabled = false;
      })
      .catch(function (err) {
        verifyStatus.className = "verify-status unverified";
        verifyStatus.textContent = "Erreur : " + (err.message || "requête échouée");
        verifyBtn.disabled = false;
      });
  });

  function escapeHtml(str) {
    if (!str) return "";
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function escapeAttr(str) {
    if (!str) return "";
    return str.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }
})();
