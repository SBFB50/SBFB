// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * `/approvals` — Sprint 83 Phase L1 — la file d'approbation HITL rendue par le
 * CHROME HÔTE (couche ii du modèle D4 : le shell navigateur de confiance).
 *
 * Invariant cardinal (§14.4) : une approbation humaine est RENDUE et signée
 * par le chrome hôte, JAMAIS par l'iframe non fiable. Cette page vit dans le
 * shell hôte : une app dans son iframe peut IMITER cet écran, mais elle ne peut
 * ni lire cette file ni transmettre une décision.
 *
 * CE QUI TIENT CET INVARIANT, exactement (correctif review r2 R5) : l'allowlist
 * FERMÉE des verbes du bridge — 16 verbes, aucun d'approbation, épinglée par
 * `web/src/bridge/__tests__/authority.test.ts` — et l'absence
 * d'`allow-same-origin` sur le sandbox, qui empêche l'iframe d'atteindre le DOM
 * de cette page. `BLOB_SERVE_CSP` scelle l'origine, le réseau et les frames du
 * contenu non fiable ; ce n'est PAS lui qui lui retire l'autorité d'approuver.
 *
 * WYSIWYS (« what you see is what you sign », précédent WebAuthn TxConf) : le
 * tuple EXACT lié à l'approbation est affiché intégralement — run, plan, nœud,
 * effet, capacité, opération, destination, digest des données sortantes, classe,
 * coût max, principal, nonce, séquence, expiration. Une demande PENDANTE ne
 * porte PAS de verdict : celui-ci est produit par le bouton, et comme il entre
 * dans le digest, Approuver et Refuser produisent DEUX digests différents
 * (correctif review r2 R7 — afficher « Verdict : granted » à côté d'un bouton
 * « Refuser » faisait lire à l'humain une décision qu'il n'avait pas prise).
 * Le daemon (couche iii, détenteur de la clé) signera le digest du tuple
 * COMPLET, verdict inclus ; toute mutation le recalcule et invalide
 * l'approbation (garantie côté moteur `hitl::verify_binding`).
 *
 * ## Ce que cette page ne fait PAS encore (à lire avant de la croire câblée)
 *
 * Aucune décision ne quitte le navigateur en L1 : la reliure (POST au daemon,
 * vérification de signature, alimentation de la couche 7) est la phase qui
 * livre la surface run. L'affordance, son accessibilité et la possession
 * HÔTE de la file sont ce que L1 livre ; le Centre des Runs plein est la
 * Phase V. Les libellés le disent explicitement plutôt que de laisser croire
 * à un acte accompli.
 *
 * Accessibilité (exigence L1.4, pas de dark-pattern d'approbation) : focus géré
 * (au montage puis après chaque décision), aucun keyboard-trap (boutons
 * standard), AUCUN type-to-confirm (approuver = une activation, pas de saisie),
 * AUCUN délai imposé (les boutons sont actifs immédiatement).
 */

import { useEffect, useRef, useState } from "react";
import { ShieldCheck } from "lucide-react";

import {
  getHostQueue,
  removeFromHostQueue,
  type HitlRequest,
  type PendingApproval,
} from "./approvalsQueue";

/**
 * Libellés des 14 champs de la DEMANDE (le 15e, le verdict, n'existe qu'après
 * la décision). `Record<keyof HitlRequest, string>` : le
 * COMPILATEUR exige une entrée par champ, donc un champ ajouté au miroir sans
 * libellé casse le build au lieu de disparaître silencieusement de l'écran que
 * l'humain est censé vérifier (correctif review D16b).
 */
const FIELD_LABELS: Record<keyof HitlRequest, string> = {
  runId: "Run",
  planDigest: "Plan",
  nodeInstanceId: "Nœud",
  effectId: "Effet",
  capabilityRef: "Capacité",
  operation: "Opération",
  destination: "Destination",
  outgoingDigest: "Digest sortant",
  effectClass: "Classe d'effet",
  // Pas d'unité inventée ici : `max_cost: u64` est un entier sans unité côté
  // Rust, et cet écran est celui où l'humain vérifie la préimage exacte.
  maxCost: "Coût max",
  principal: "Principal",
  nonce: "Nonce",
  seq: "Séquence",
  expiresAtMs: "Expiration",
};

/** Ordre voulu à l'écran — l'ordre du tuple Rust. */
const DISPLAY_ORDER: Array<keyof HitlRequest> = [
  "runId",
  "planDigest",
  "nodeInstanceId",
  "effectId",
  "capabilityRef",
  "operation",
  "destination",
  "outgoingDigest",
  "effectClass",
  "maxCost",
  "principal",
  "nonce",
  "seq",
  "expiresAtMs",
];

/**
 * Ce qui est RENDU : dérivé des clés de `FIELD_LABELS`, donc un champ ajouté au
 * miroir apparaît forcément à l'écran (correctif review r2 P2-16 — une liste
 * tenue à la main pouvait taire un champ tout en gardant son libellé).
 *
 * `DISPLAY_ORDER` ne fait que TRIER, il ne filtre pas : un champ absent de cet
 * ordre est poussé en QUEUE, jamais omis, et jamais promu en tête par un
 * `indexOf` valant -1. La complétude vient de `Object.keys`, l'ordre est du
 * confort de lecture.
 */
const FIELD_ORDER: Array<keyof HitlRequest> = (
  Object.keys(FIELD_LABELS) as Array<keyof HitlRequest>
).sort((a, b) => displayRank(a) - displayRank(b));

function displayRank(field: keyof HitlRequest): number {
  const rank = DISPLAY_ORDER.indexOf(field);
  return rank < 0 ? Number.MAX_SAFE_INTEGER : rank;
}

export default function Approvals() {
  const [queue, setQueue] = useState<readonly PendingApproval[]>(() =>
    getHostQueue(),
  );
  const [lastDecision, setLastDecision] = useState<string | null>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const statusRef = useRef<HTMLDivElement>(null);

  // Focus géré : au montage, le titre de la file reçoit le focus, de sorte
  // qu'un lecteur d'écran annonce le contexte d'approbation. Pas de piège :
  // la tabulation continue normalement vers les boutons.
  useEffect(() => {
    headingRef.current?.focus();
  }, []);

  function decide(requestId: string, decision: "approve" | "reject") {
    // Retire l'approbation traitée de la file host-owned (module + état local).
    setQueue(removeFromHostQueue(requestId));
    // UNE seule sémantique du refus, et c'est celle du moteur (correctif review
    // r2 R7) : `decision` est DANS le tuple signé, donc un refus est une
    // décision SIGNÉE comme un accord — c'est précisément ce qui l'empêche
    // d'être rejoué en accord (`hitl::a_refusal_never_decides_a_ref`) et ce qui
    // en laisse une trace auditable (axe Répudiation de l'asset A15). Un refus
    // « non signé » ne serait pas distinguable d'une absence de réponse.
    // Le libellé dit UNIQUEMENT ce qui s'est produit (correctif review r3
    // L1-R3-07) : `decide()` retire la demande de la file et affiche ce message.
    // Rien n'est transmis, rien n'est persisté — ni localStorage, ni POST, ni
    // journal. Une version précédente affirmait « enregistré localement » puis
    // « ce qui le rend auditable » : deux faits qui n'existent pas, dont le
    // second promettait une trace que l'axe Répudiation de A15 n'a pas encore.
    setLastDecision(
      decision === "approve"
        ? "Accord pris dans cet écran. Rien n'est transmis ni conservé : la signature hôte et le journal d'approbations ne sont pas encore câblés. Le nœud signera ce tuple, verdict inclus."
        : "Refus pris dans cet écran. Rien n'est transmis ni conservé : la signature hôte et le journal d'approbations ne sont pas encore câblés. Un refus se signe comme un accord — deux verdicts, deux digests.",
    );
    // Le focus va sur le PORTEUR du message, pas sur le titre (correctif review
    // r2, résidu a11y) : déplacer le focus dans le même tour qu'une mise à jour
    // de région `polite` fait couramment abandonner l'annonce en attente par
    // NVDA/JAWS — l'opérateur AT n'entendrait jamais le seul retour sur son acte
    // d'autorité, la carte disparaissant pour un accord comme pour un refus.
    statusRef.current?.focus();
  }

  return (
    <div className="space-y-6" data-testid="hitl-host-chrome">
      <header className="space-y-2">
        <div className="flex items-center gap-2">
          <ShieldCheck className="h-5 w-5 text-emerald-400" aria-hidden="true" />
          <h1
            ref={headingRef}
            tabIndex={-1}
            data-testid="hitl-heading"
            className="text-xl font-bold focus-visible:outline focus-visible:outline-2 focus-visible:outline-white/40"
          >
            Approbations en attente
          </h1>
        </div>
        <p className="max-w-2xl text-sm text-white/60">
          Ces demandes sont rendues par le chrome hôte de confiance. Vérifie le
          tuple exact avant d&apos;approuver — c&apos;est ce que le nœud
          signera. Une app dans son iframe peut imiter cet écran, mais elle ne
          peut ni lire cette file ni décider à ta place : seule cette page du
          shell rend ces demandes. La transmission au nœud n&apos;est pas encore
          câblée.
        </p>
        <p className="max-w-2xl text-sm text-white/60">
          Ton verdict fait partie du tuple signé :{" "}
          <strong className="font-semibold text-white/80">
            approuver et refuser produisent deux digests différents
          </strong>
          . Un refus ne peut donc pas être rejoué comme un accord.
        </p>
      </header>

      <div
        ref={statusRef}
        tabIndex={-1}
        role="status"
        aria-live="polite"
        aria-atomic="true"
        data-testid="hitl-decision-status"
        className="text-sm text-white/50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-white/40"
      >
        {lastDecision}
      </div>

      {queue.length === 0 ? (
        <div className="glass-card max-w-md p-6" data-testid="hitl-empty">
          <h2 className="mb-1 font-bold">Aucune approbation en attente</h2>
          <p className="text-sm text-white/50">
            Les demandes d&apos;effet nécessitant un accord humain
            apparaîtront ici lorsque la surface run sera câblée — aucun
            producteur ne les émet encore.
          </p>
        </div>
      ) : (
        <ul className="space-y-4" data-testid="hitl-queue">
          {queue.map((approval) => (
            <ApprovalCard
              key={approval.requestId}
              approval={approval}
              onDecide={decide}
            />
          ))}
        </ul>
      )}
    </div>
  );
}

function ApprovalCard({
  approval,
  onDecide,
}: {
  approval: PendingApproval;
  onDecide: (requestId: string, decision: "approve" | "reject") => void;
}) {
  const { request, requestId } = approval;
  const labelId = `hitl-${requestId}`;

  return (
    <li>
      <section
        role="group"
        aria-labelledby={labelId}
        data-testid="hitl-approval-card"
        data-request-id={requestId}
        className="glass-card space-y-4 p-5"
      >
        <h2 id={labelId} className="font-bold">
          {request.operation} → {request.destination}
        </h2>

        <dl className="grid grid-cols-1 gap-x-6 gap-y-1 sm:grid-cols-2">
          {FIELD_ORDER.map((field) => (
            <div key={field} className="flex justify-between gap-3 text-sm">
              <dt className="text-white/50">{FIELD_LABELS[field]}</dt>
              {/* `break-all` plutôt que `truncate` : un digest tronqué ne peut
                  pas être vérifié par l'humain, et un `title` n'est pas lisible
                  au clavier ni par une AT. */}
              <dd
                className="break-all text-right font-mono text-white/80"
                data-field={field}
              >
                {field === "expiresAtMs" ? (
                  <ExpiryValue raw={request.expiresAtMs} />
                ) : (
                  String(request[field])
                )}
              </dd>
            </div>
          ))}
        </dl>

        <div className="flex gap-3">
          <button
            type="button"
            data-testid="hitl-approve"
            onClick={() => onDecide(requestId, "approve")}
            className="rounded-md bg-emerald-500/90 px-4 py-2 text-sm font-semibold text-black hover:bg-emerald-400 focus-visible:outline focus-visible:outline-2 focus-visible:outline-emerald-300"
          >
            Approuver
          </button>
          <button
            type="button"
            data-testid="hitl-reject"
            onClick={() => onDecide(requestId, "reject")}
            className="rounded-md border border-white/20 px-4 py-2 text-sm font-semibold text-white/80 hover:bg-white/10 focus-visible:outline focus-visible:outline-2 focus-visible:outline-white/40"
          >
            Refuser
          </button>
        </div>
      </section>
    </li>
  );
}

/**
 * Borne ECMAScript d'un instant représentable : ±8,64e15 ms autour de l'epoch.
 * Au-delà, `new Date(ms).toISOString()` JETTE un `RangeError`.
 */
const MAX_TIME_VALUE_MS = 8.64e15;

/**
 * L'expiration, rendue lisible SANS pouvoir casser la page (correctif review r2
 * P2-18).
 *
 * Le champ est un `i64` côté Rust, et sa valeur extrême (la borne du type ;
 * aucune sémantique « sans expiration » n'existe côté moteur) dépasse la borne
 * des dates représentables : la version initiale faisait alors jeter
 * `toISOString()` PENDANT le rendu, ce qui remontait à la frontière d'erreur de
 * route et rendait `/approvals` entièrement inutilisable — approbations saines
 * comprises, donc tout run human-gated calé.
 *
 * La préimage BRUTE est donc TOUJOURS affichée, et l'ISO seulement à côté, en
 * confort de lecture. Ce n'est pas cosmétique : les octets bruts sont ce que
 * l'humain signe, et ce champ était le seul des 14 à ne pas être rendu verbatim.
 */

/**
 * Texte d'une valeur d'expiration, sans jamais invoquer de méthode sur elle.
 *
 * `String(value)` déclenche `Symbol.toPrimitive`/`valueOf`/`toString` — donc un
 * objet hostile fait jeter le rendu, exactement le mode d'échec que le garde
 * était censé supprimer (ex-review r4 L1-R4-F01 : le jet avait été DÉPLACÉ de
 * `.trim()` vers `Number()`/`String()`, pas supprimé). On ne convertit que les
 * primitives pour lesquelles la conversion est totale.
 */
function rawExpiryText(raw: unknown): string {
  if (raw === null || raw === undefined) return "(absente)";
  switch (typeof raw) {
    case "string":
      return raw;
    case "number":
    case "boolean":
    case "bigint":
      return String(raw);
    default:
      return `(valeur ${typeof raw} hors contrat)`;
  }
}

function ExpiryValue({ raw }: { raw: unknown }) {
  const text = rawExpiryText(raw);
  // Le gate de TYPE vient en premier, et il porte sur les deux branches. Un
  // champ du tuple est une chaîne par contrat ; toute autre forme vient d'une
  // source non conforme et n'est PAS convertie en date : `false` devenait
  // « 1970-01-01T00:00:00.000Z », une expiration plausible et FAUSSE sur
  // l'écran d'autorité (ex-review r4 L1-R4-F02).
  if (typeof raw !== "string") {
    return <span data-expiry-format="raw">{text}</span>;
  }
  const trimmed = raw.trim();
  const ms = Number(trimmed);
  // L'ISO n'est proposé que si la conversion est FIDÈLE dans les deux sens.
  // `"1e3"`, `"0x10"` et `"9223372036854775807"` (perte de précision au-delà de
  // 2^53) ne le sont pas : ils restent en octets bruts plutôt que d'afficher un
  // nombre que la préimage ne contient pas.
  const faithful =
    trimmed !== "" &&
    Number.isFinite(ms) &&
    String(ms) === trimmed &&
    Math.abs(ms) <= MAX_TIME_VALUE_MS;
  if (!faithful) {
    return <span data-expiry-format="raw">{text}</span>;
  }
  const iso = new Date(ms).toISOString();
  return (
    <span data-expiry-format="iso">
      {raw} (<time dateTime={iso}>{iso}</time>)
    </span>
  );
}

// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Approvals;
