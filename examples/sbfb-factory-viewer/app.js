// SPDX-License-Identifier: AGPL-3.0-or-later
/* Factory Viewer — app SBFB sandboxee, lecture seule via bridge. */

(function () {
  "use strict";

  var bridge = new SBFBBridge({ timeout: 8000, heartbeatInterval: 0 });
  var apps = [];
  var currentFilter = "all";

  var grid = document.getElementById("app-grid");
  var searchInput = document.getElementById("search-input");
  var viewList = document.getElementById("view-list");
  var viewDetail = document.getElementById("view-detail");
  var detailContent = document.getElementById("detail-content");
  var backBtn = document.getElementById("back-btn");
  var filterBtns = document.querySelectorAll(".filter-btn");

  function escapeHtml(str) {
    var div = document.createElement("div");
    div.textContent = str;
    return div.innerHTML;
  }

  function truncate(str, len) {
    if (!str) return "";
    return str.length > len ? str.slice(0, len) + "..." : str;
  }

  function renderAppCard(app) {
    var statusClass = app.published
      ? "app-card-status-published"
      : "app-card-status-dev";
    var statusText = app.published ? "● Publiée" : "○ Dev";

    return (
      '<button type="button" class="app-card" data-name="' + escapeHtml(app.name) + '" aria-label="' + escapeHtml(app.name) + '">' +
      '<div class="app-card-name">' + escapeHtml(app.name) + "</div>" +
      '<div class="app-card-desc">' + escapeHtml(app.description || "") + "</div>" +
      '<div class="app-card-meta">' +
      '<span class="app-card-version">v' + escapeHtml(app.version || "0.0.0") + "</span>" +
      '<span class="app-card-category">' + escapeHtml(app.category || "") + "</span>" +
      '<span class="' + statusClass + '">' + statusText + "</span>" +
      "</div></button>"
    );
  }

  function renderGrid() {
    var query = (searchInput.value || "").toLowerCase();
    var filtered = apps.filter(function (app) {
      if (currentFilter === "published" && !app.published) return false;
      if (currentFilter === "dev" && app.published) return false;
      if (query && app.name.toLowerCase().indexOf(query) === -1) return false;
      return true;
    });

    if (filtered.length === 0) {
      grid.innerHTML = '<p class="empty-state">Aucune app trouvée.</p>';
      return;
    }

    grid.innerHTML = filtered.map(renderAppCard).join("");

    var cards = grid.querySelectorAll(".app-card");
    for (var i = 0; i < cards.length; i++) {
      cards[i].addEventListener("click", function () {
        showDetail(this.getAttribute("data-name"));
      });
    }
  }

  function showDetail(name) {
    var app = apps.find(function (a) { return a.name === name; });
    if (!app) return;

    viewList.hidden = true;
    viewDetail.hidden = false;

    var html =
      '<div class="detail-header">' +
      '<div class="detail-name">' + escapeHtml(app.name) + "</div>" +
      '<div class="detail-desc">' + escapeHtml(app.description || "") + "</div>" +
      '<div class="detail-meta">' +
      "<span>Version : v" + escapeHtml(app.version || "0.0.0") + "</span>" +
      "<span>Catégorie : " + escapeHtml(app.category || "") + "</span>" +
      "<span>" + (app.published ? "● Publiée" : "○ Dev") + "</span>" +
      "</div></div>";

    detailContent.innerHTML = html + '<p class="empty-state">Chargement de la preuve...</p>';

    bridge.getProofCard(app.name).then(function (proof) {
      if (proof && proof.commit_source) {
        var verifiedClass = proof.verified ? "proof-verified" : "proof-unverified";
        var verifiedText = proof.verified ? "✓ Vérifié" : "Non vérifié";

        html +=
          '<div class="proof-card">' +
          '<div class="proof-card-title">Preuve de provenance</div>' +
          '<div class="proof-grid">' +
          '<span class="proof-label">Commit source</span>' +
          '<span class="proof-value">' + escapeHtml(proof.commit_source) + "</span>" +
          '<span class="proof-label">Hash archive</span>' +
          '<span class="proof-value">' + escapeHtml(truncate(proof.archive_hash, 48)) + "</span>" +
          '<span class="proof-label">Signataire</span>' +
          '<span class="proof-value">' + escapeHtml(truncate(proof.signer_pubkey, 48)) + "</span>" +
          '<span class="proof-label">Verdict</span>' +
          '<span class="' + verifiedClass + '">' + verifiedText + "</span>" +
          "</div></div>";
      } else {
        html += '<p class="empty-state">Aucune preuve de provenance disponible.</p>';
      }

      html +=
        '<div class="redirect-banner">' +
        "Pour créer, modifier ou publier cette app → Factory Operator" +
        "</div>";

      detailContent.innerHTML = html;
    }).catch(function () {
      detailContent.innerHTML = html +
        '<p class="empty-state">Impossible de charger la preuve.</p>' +
        '<div class="redirect-banner">' +
        "Pour créer, modifier ou publier cette app → Factory Operator" +
        "</div>";
    });
  }

  function showList() {
    viewList.hidden = false;
    viewDetail.hidden = true;
  }

  backBtn.addEventListener("click", showList);

  searchInput.addEventListener("input", renderGrid);

  for (var i = 0; i < filterBtns.length; i++) {
    filterBtns[i].addEventListener("click", function () {
      for (var j = 0; j < filterBtns.length; j++) {
        filterBtns[j].classList.remove("active");
      }
      this.classList.add("active");
      currentFilter = this.getAttribute("data-filter");
      renderGrid();
    });
  }

  bridge.getBrowseList().then(function (result) {
    apps = (result && result.projects) || [];
    renderGrid();
  }).catch(function () {
    grid.innerHTML = '<p class="empty-state">Impossible de charger les apps du réseau.</p>';
  });
})();
