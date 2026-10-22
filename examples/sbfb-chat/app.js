// SPDX-License-Identifier: AGPL-3.0-or-later
//
// SBFB Chat — end-to-end encrypted messaging (chat v1, ADR_CHAT_V1).
// Every key lives in the local daemon; this page only ever sees the
// plaintext it displays. The sandboxed iframe blocks form submission,
// so every action is a div + button + click handler.

(function () {
  "use strict";

  var bridge = null;
  var me = null;
  var conversations = [];
  var current = null;
  var mailbox = null;

  function $(id) {
    return document.getElementById(id);
  }

  var el = {
    dot: $("identity-dot"),
    label: $("identity-label"),
    notice: $("notice"),
    onboarding: $("onboarding"),
    profileName: $("profile-name"),
    profileSave: $("profile-save"),
    layout: $("layout"),
    newTitle: $("new-conv-title"),
    newBtn: $("new-conv-btn"),
    joinLink: $("join-link"),
    joinBtn: $("join-btn"),
    convList: $("conv-list"),
    mailboxBtn: $("mailbox-btn"),
    mailboxState: $("mailbox-state"),
    chatEmpty: $("chat-empty"),
    chatInner: $("chat-inner"),
    chatTitle: $("chat-title"),
    inviteBtn: $("invite-btn"),
    inviteBox: $("invite-box"),
    inviteLink: $("invite-link"),
    inviteCopy: $("invite-copy"),
    inviteExpiry: $("invite-expiry"),
    membersSummary: $("members-summary"),
    memberList: $("member-list"),
    messages: $("messages"),
    composer: $("composer-text"),
    send: $("send-btn"),
  };

  // --- Small helpers ---------------------------------------------------

  function notify(text, isError) {
    el.notice.textContent = text || "";
    el.notice.className = "notice" + (text ? (isError ? " error" : " info") : "");
  }

  function explain(err) {
    var msg = (err && err.message) || String(err);
    if (/not authorized/i.test(msg)) {
      return "La messagerie n'est pas autorisée pour cette app : acceptez la demande affichée au-dessus de l'app.";
    }
    if (/timed out|timeout/i.test(msg)) {
      return "Le nœud ne répond pas. Vérifiez qu'il est démarré.";
    }
    return msg;
  }

  function fail(err) {
    notify(explain(err), true);
  }

  function formatTime(secs) {
    try {
      return new Date(secs * 1000).toLocaleString("fr-FR", {
        day: "numeric",
        month: "short",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch (e) {
      return "";
    }
  }

  function clear(node) {
    while (node.firstChild) node.removeChild(node.firstChild);
  }

  function text(tag, value, cls) {
    var n = document.createElement(tag);
    n.textContent = value;
    if (cls) n.className = cls;
    return n;
  }

  function busy(button, on) {
    button.disabled = on;
    button.setAttribute("aria-busy", on ? "true" : "false");
  }

  // --- Identity --------------------------------------------------------

  function showIdentity() {
    el.dot.className = "identity-dot online";
    el.label.textContent = (me.name || "Sans nom") + " · " + me.fingerprint;
    el.label.title = "Votre empreinte de sécurité : comparez-la de vive voix avec vos contacts.";
  }

  function loadIdentity() {
    return bridge.chatIdentity().then(function (id) {
      me = id;
      showIdentity();
      if (!me.name) {
        el.onboarding.hidden = false;
        el.layout.hidden = true;
        el.profileName.focus();
      } else {
        el.onboarding.hidden = true;
        el.layout.hidden = false;
      }
    });
  }

  el.profileSave.addEventListener("click", function () {
    var name = el.profileName.value.trim();
    if (!name) {
      notify("Choisissez un nom.", true);
      return;
    }
    busy(el.profileSave, true);
    bridge
      .chatSetProfile(name)
      .then(function () {
        notify("");
        return loadIdentity();
      })
      .then(refreshAll)
      .catch(fail)
      .then(function () {
        busy(el.profileSave, false);
      });
  });

  // --- Conversations ---------------------------------------------------

  function renderConversations() {
    clear(el.convList);
    if (conversations.length === 0) {
      el.convList.appendChild(text("li", "Aucune conversation.", "muted small"));
      return;
    }
    conversations.forEach(function (c) {
      var li = document.createElement("li");
      var b = document.createElement("button");
      b.type = "button";
      b.className = "conv-item" + (current === c.id ? " active" : "");
      if (current === c.id) b.setAttribute("aria-current", "true");
      b.appendChild(text("span", c.title || "Sans titre", "conv-title"));
      var meta = c.removed ? "vous n'en faites plus partie" : c.members.length + " membre(s)";
      b.appendChild(text("span", meta, "conv-meta small muted"));
      b.addEventListener("click", function () {
        openConversation(c.id);
      });
      li.appendChild(b);
      el.convList.appendChild(li);
    });
  }

  function loadConversations() {
    return bridge.chatList().then(function (list) {
      conversations = list || [];
      renderConversations();
      if (current) renderHeader();
    });
  }

  function find(id) {
    for (var i = 0; i < conversations.length; i++) {
      if (conversations[i].id === id) return conversations[i];
    }
    return null;
  }

  el.newBtn.addEventListener("click", function () {
    var title = el.newTitle.value.trim();
    if (!title) {
      notify("Donnez un titre à la conversation.", true);
      return;
    }
    busy(el.newBtn, true);
    bridge
      .chatCreate(title)
      .then(function (r) {
        el.newTitle.value = "";
        notify("Conversation créée. Invitez quelqu'un avec le bouton « Inviter ».");
        return loadConversations().then(function () {
          openConversation(r.id);
        });
      })
      .catch(fail)
      .then(function () {
        busy(el.newBtn, false);
      });
  });

  el.joinBtn.addEventListener("click", function () {
    var link = el.joinLink.value.trim();
    if (link.indexOf("sbfb-chat:") !== 0) {
      notify("Ce n'est pas un lien d'invitation SBFB Chat.", true);
      return;
    }
    busy(el.joinBtn, true);
    bridge
      .chatAccept(link)
      .then(function (r) {
        el.joinLink.value = "";
        notify("Vous avez rejoint la conversation. Les messages arrivent dès que l'admin est joignable.");
        return loadConversations().then(function () {
          openConversation(r.id);
        });
      })
      .catch(fail)
      .then(function () {
        busy(el.joinBtn, false);
      });
  });

  // --- One conversation ------------------------------------------------

  function renderHeader() {
    var c = find(current);
    if (!c) return;
    el.chatTitle.textContent = c.title || "Sans titre";
    el.inviteBtn.hidden = !c.is_admin || c.removed;
    el.send.disabled = c.removed;
    el.composer.disabled = c.removed;
    el.membersSummary.textContent = "Membres (" + c.members.length + ")";
    clear(el.memberList);
    c.members.forEach(function (m) {
      var li = document.createElement("li");
      li.appendChild(text("span", m.name || "Sans nom", "member-name"));
      if (m.admin) li.appendChild(text("span", "admin", "badge"));
      if (m.id === me.id) li.appendChild(text("span", "vous", "badge"));
      li.appendChild(text("span", m.fingerprint, "mono small muted"));
      if (c.is_admin && m.id !== me.id && !c.removed) {
        var rm = document.createElement("button");
        rm.type = "button";
        rm.className = "btn btn-small btn-danger";
        rm.textContent = "Retirer";
        rm.setAttribute("aria-label", "Retirer " + (m.name || "ce membre"));
        rm.addEventListener("click", function () {
          removeMember(c.id, m);
        });
        li.appendChild(rm);
      }
      el.memberList.appendChild(li);
    });
  }

  function appendMessage(m) {
    var li = document.createElement("li");
    li.className = "message" + (m.mine ? " mine" : "");
    var head = document.createElement("div");
    head.className = "message-head small muted";
    head.appendChild(text("span", m.mine ? "Vous" : m.from_name || "Inconnu"));
    head.appendChild(text("span", " · " + formatTime(m.sent_at)));
    li.appendChild(head);
    li.appendChild(text("p", m.body, "message-body"));
    el.messages.appendChild(li);
  }

  function loadMessages() {
    var id = current;
    return bridge.chatMessages(id).then(function (list) {
      if (id !== current) return;
      clear(el.messages);
      (list || []).forEach(appendMessage);
      el.messages.scrollTop = el.messages.scrollHeight;
    });
  }

  function openConversation(id) {
    current = id;
    el.inviteBox.hidden = true;
    el.chatEmpty.hidden = true;
    el.chatInner.hidden = false;
    renderConversations();
    renderHeader();
    loadMessages().catch(fail);
    el.composer.focus();
  }

  function sendCurrent() {
    var body = el.composer.value;
    if (!body.trim() || !current) return;
    busy(el.send, true);
    bridge
      .chatSend(current, body)
      .then(function () {
        el.composer.value = "";
        notify("");
      })
      .catch(fail)
      .then(function () {
        busy(el.send, false);
        el.composer.focus();
      });
  }

  el.send.addEventListener("click", sendCurrent);
  el.composer.addEventListener("keydown", function (e) {
    if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
      e.preventDefault();
      sendCurrent();
    }
  });

  el.inviteBtn.addEventListener("click", function () {
    busy(el.inviteBtn, true);
    bridge
      .chatInvite(current)
      .then(function (r) {
        el.inviteLink.value = r.invite;
        el.inviteExpiry.textContent = "Valable jusqu'au " + formatTime(r.expires_at) + ".";
        el.inviteBox.hidden = false;
        el.inviteLink.focus();
        el.inviteLink.select();
      })
      .catch(fail)
      .then(function () {
        busy(el.inviteBtn, false);
      });
  });

  el.inviteCopy.addEventListener("click", function () {
    el.inviteLink.focus();
    el.inviteLink.select();
    var copied = false;
    try {
      copied = document.execCommand("copy");
    } catch (e) {
      copied = false;
    }
    notify(copied ? "Lien copié." : "Copie impossible ici : le lien est sélectionné, copiez-le avec Ctrl+C.");
  });

  // No confirm(): modal dialogs are blocked in the sandboxed iframe. The
  // button names who is removed.
  function removeMember(convId, member) {
    bridge
      .chatRemove(convId, member.id)
      .then(function () {
        notify((member.name || "Le membre") + " a été retiré. Les clés ont changé : il ne lit plus les nouveaux messages.");
        return loadConversations();
      })
      .catch(fail);
  }

  // --- Mailbox ---------------------------------------------------------

  var STATUS = { pending: "en cours", kept: "gardée", failed: "échec, nouvel essai bientôt" };

  function renderMailbox() {
    if (!mailbox) return;
    el.mailboxBtn.textContent = mailbox.enabled ? "Désactiver" : "Activer";
    el.mailboxBtn.setAttribute("aria-pressed", mailbox.enabled ? "true" : "false");
    if (!mailbox.servers || mailbox.servers.length === 0) {
      el.mailboxState.textContent = "Aucun nœud boîte aux lettres configuré.";
      el.mailboxBtn.disabled = true;
      return;
    }
    el.mailboxBtn.disabled = false;
    if (!mailbox.enabled) {
      el.mailboxState.textContent = "Désactivée.";
      return;
    }
    var kept = mailbox.conversations.filter(function (c) {
      return c.status === "kept";
    }).length;
    el.mailboxState.textContent =
      "Activée — " + kept + "/" + mailbox.conversations.length + " conversation(s) gardée(s).";
    var detail = mailbox.conversations.map(function (c) {
      var conv = find(c.id);
      return (conv ? conv.title : c.id.slice(0, 8)) + " : " + (STATUS[c.status] || c.status);
    });
    el.mailboxState.title = detail.join("\n");
  }

  function loadMailbox() {
    return bridge.chatMailbox().then(function (m) {
      mailbox = m;
      renderMailbox();
    });
  }

  el.mailboxBtn.addEventListener("click", function () {
    if (!mailbox) return;
    busy(el.mailboxBtn, true);
    bridge
      .chatMailbox(!mailbox.enabled)
      .then(function (m) {
        mailbox = m;
        renderMailbox();
      })
      .catch(fail)
      .then(function () {
        busy(el.mailboxBtn, false);
        renderMailbox();
      });
  });

  // --- Live updates ----------------------------------------------------

  function onChat(ev) {
    if (!ev || !ev.type) return;
    if (ev.type === "message") {
      if (ev.conversation === current) {
        appendMessage(ev);
        el.messages.scrollTop = el.messages.scrollHeight;
      } else {
        var c = find(ev.conversation);
        notify("Nouveau message dans « " + (c ? c.title : "une conversation") + " ».");
      }
      return;
    }
    if (ev.type === "members" || ev.type === "removed") {
      loadConversations().catch(fail);
      if (ev.type === "removed" && ev.conversation === current) {
        notify("Vous avez été retiré de cette conversation.", true);
      }
    }
  }

  // --- Boot ------------------------------------------------------------

  function refreshAll() {
    return Promise.all([loadConversations(), loadMailbox()]).catch(fail);
  }

  try {
    bridge = new SBFBBridge({ timeout: 15000 });
  } catch (e) {
    el.dot.className = "identity-dot offline";
    el.label.textContent = "Hors du réseau SBFB";
    return;
  }

  bridge.onChat(onChat);
  loadIdentity()
    .then(function () {
      if (me && me.name) return refreshAll();
    })
    .catch(function (err) {
      el.dot.className = "identity-dot offline";
      el.label.textContent = "Indisponible";
      fail(err);
    });
  // Mailbox status moves on its own (deposits, daily refresh).
  setInterval(function () {
    if (me && me.name) loadMailbox().catch(function () {});
  }, 30000);
})();
