// SPDX-License-Identifier: AGPL-3.0-or-later
//
// SBFB Compute Tester — Sprint 76 Phase H.
//
// Proves the local compute chain end to end:
//   app -> bridge (task_submit) -> daemon /api/v1/tasks/submit
//       -> on-demand local worker (claim + Ollama) -> result
//       -> daemon /result -> bridge (task_result poll) -> app.
//
// The app stays node-agnostic: it sends only {prompt, model,
// task_type}. The host bridge injects the local project_id so the
// node's own worker claims the task. The result is not pushed back —
// the app polls `getTaskResult` until it is ready (a daemon push
// channel backed by iroh-docs subscribe is scoped for S77).

(function () {
  "use strict";

  var bridge = new SBFBBridge();

  var promptEl = document.getElementById("prompt");
  var modelEl = document.getElementById("model");
  var submitEl = document.getElementById("submit");
  var stateEl = document.getElementById("state");
  var resultEl = document.getElementById("result");
  var metaEl = document.getElementById("meta");

  var POLL_INTERVAL_MS = 1500;
  var MAX_WAIT_MS = 120000; // 2 min before declaring a timeout

  function setState(text, kind) {
    stateEl.textContent = text;
    stateEl.className = "state" + (kind ? " " + kind : "");
  }

  function showResult(text) {
    resultEl.textContent = text;
    resultEl.style.display = "block";
  }

  function sleep(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms); });
  }

  async function run() {
    var prompt = (promptEl.value || "").trim();
    var model = (modelEl.value || "").trim();
    if (!prompt) { setState("Entrez une demande.", "err"); return; }
    if (!model) { setState("Entrez un modele.", "err"); return; }

    submitEl.disabled = true;
    resultEl.style.display = "none";
    metaEl.textContent = "";
    setState("Soumission…");

    var started = Date.now();
    var taskId;
    try {
      var sub = await bridge.submitTask({
        prompt: prompt,
        model: model,
        task_type: "inference",
      });
      taskId = sub.task_id;
    } catch (e) {
      setState("Echec de la soumission : " + (e && e.message ? e.message : e), "err");
      submitEl.disabled = false;
      return;
    }

    metaEl.textContent = "task_id : " + taskId;
    setState("En cours… (un worker GPU local execute la tache)");

    while (Date.now() - started < MAX_WAIT_MS) {
      await sleep(POLL_INTERVAL_MS);
      var poll;
      try {
        poll = await bridge.getTaskResult(taskId);
      } catch (e) {
        // Transient bridge/HTTP hiccup — keep polling until the
        // overall deadline.
        continue;
      }
      if (poll && poll.ready) {
        var secs = ((Date.now() - started) / 1000).toFixed(1);
        setState("Termine en " + secs + " s.", "ok");
        showResult(poll.result_text);
        metaEl.textContent = "task_id : " + taskId +
          (poll.result_hash ? "  ·  hash : " + poll.result_hash : "");
        submitEl.disabled = false;
        return;
      }
      var elapsed = ((Date.now() - started) / 1000).toFixed(0);
      setState("En cours… " + elapsed + " s");
    }

    setState(
      "Delai depasse (aucun resultat en 2 min). Le worker a-t-il claime la tache ? Ollama est-il demarre ?",
      "err",
    );
    submitEl.disabled = false;
  }

  submitEl.addEventListener("click", run);
})();
