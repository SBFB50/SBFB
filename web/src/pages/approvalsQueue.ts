// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase L1 — la file d'approbation HITL, host-owned.
 *
 * Cet état vit dans le SHELL HÔTE, jamais dans une iframe d'app : c'est
 * l'invariant §14.4 (« rendu et signé par le chrome hôte »). Le module est
 * séparé de la page pour que la file survive au démontage/remontage de
 * `Approvals` ET à la fermeture d'une iframe (l'iframe ne possède pas cet
 * état).
 *
 * VIDE PAR DÉFAUT (correctif review P1-7). La première version amorçait ici
 * une approbation de démonstration, sans aucun garde : un build réel affichait
 * donc à l'opérateur une demande d'autorité plausible et FAUSSE, ne
 * correspondant à aucun run — exactement la classe que ce projet interdit
 * (« 0 verdict calculé UI », « diff = vérité Rust »), et le pire endroit pour
 * la commettre puisqu'un modèle HITL ne doit jamais entraîner un opérateur sur
 * une fausse approbation. Les fixtures sont désormais INJECTÉES par les tests :
 * `window.__SBFB_HITL_QUEUE` (e2e, via `page.addInitScript` — précédent
 * `web/e2e/fixtures.ts`) ou `__seedHostQueueForTest()` (vitest).
 *
 * Bornes honnêtes de cette file : elle survit à la mort d'une iframe (elle vit
 * dans le document hôte) mais PAS à un rechargement du shell — aucune
 * persistance. La file durable est celle du DAEMON (surface loopback), non
 * câblée en L1 : cf. threat model §19.4, borne 5. Le renvoi est vers la BORNE
 * et non vers un numéro de phase, qui se re-route (ce sprint l'a déjà fait deux
 * fois) et laisserait un pointeur mort.
 */

/**
 * Le tuple EXACT lié à une approbation (miroir de `HitlBinding` côté moteur,
 * `crates/sbfb-workflow-engine/src/hitl.rs`).
 *
 * Les noms de champs Rust sont la PRÉIMAGE de la signature hôte et sont gelés ;
 * ce miroir TS est en camelCase par choix de style front, donc la phase qui
 * câblera la route loopback devra porter un mapper explicite (propriété, pas
 * numéro de phase : celui-ci se re-route).
 *
 * PORTÉE EXACTE du garde-fou (correctif review r2 P2-17 — la version précédente
 * de ce paragraphe promettait davantage) : `Approvals.test.tsx` épingle la liste
 * littérale des 14 champs de la DEMANDE, et `Record<keyof HitlRequest, string>`
 * force un libellé par champ. Ces deux contrôles comparent le miroir à sa propre
 * définition TS : ils attrapent une édition TS, JAMAIS une édition Rust. La
 * parité cross-langage est tenue par la jambe `HITL-PARITY` de
 * `scripts/check-frontier-contracts.sh`, seule autorité sur « le miroir a
 * dérivé de sa source ».
 *
 * ENTIERS RUST → `string` (classe, pas instance) : les trois champs numériques
 * du tuple Rust (`max_cost: u64`, `seq: u64`, `expires_at_ms: i64`) sont des
 * chaînes ici. Deux raisons, et elles valent pour les trois : une valeur
 * dépassant 2^53 doit rester EXACTE (c'est la préimage d'une signature), et une
 * valeur extrême ne doit pas pouvoir faire jeter le rendu.
 *
 * Aucune SÉMANTIQUE n'est inventée ici (correctif review r3 L1-R3-08) : une
 * version précédente affirmait que `u64::MAX` / `i64::MAX` signifiaient « sans
 * plafond » / « sans expiration » côté Rust. C'est faux — aucune constante de ce
 * genre n'existe dans le moteur, et déclarer une convention sur l'autre moitié
 * d'un seam de préimage gelé est exactement ce qu'un miroir ne doit pas faire. Le
 * repli d'affichage reste juste pour une autre raison : toute valeur non
 * représentable comme date doit s'afficher brute plutôt que jeter.
 */
export interface HitlBinding {
  runId: string;
  planDigest: string;
  nodeInstanceId: string;
  effectId: string;
  capabilityRef: string;
  operation: string;
  destination: string;
  outgoingDigest: string;
  effectClass: string;
  maxCost: string;
  principal: string;
  nonce: string;
  seq: string;
  expiresAtMs: string;
  decision: "granted" | "refused";
}

/**
 * Une demande PENDANTE : le tuple SANS le verdict (correctif review r2 R7).
 *
 * Une demande en attente ne peut pas porter `decision`, sinon l'écran affiche
 * « Verdict : granted » à côté d'un bouton « Refuser » — une contradiction, et
 * pire : l'humain lit un verdict qu'il n'a pas donné. Le verdict est PRODUIT par
 * le bouton, et comme il entre dans le digest, Approuver et Refuser produisent
 * DEUX digests différents (côté Rust : `a_refusal_never_decides_a_ref`).
 */
export type HitlRequest = Omit<HitlBinding, "decision">;

/** Une approbation en attente dans la file host-owned. */
export interface PendingApproval {
  /**
   * Identifiant de la demande. Ce n'est PAS le digest signé : celui-ci dépend
   * du verdict, donc il n'existe qu'une fois la décision prise (deux verdicts
   * ⇒ deux digests).
   */
  requestId: string;
  /** Le tuple affiché, WYSIWYS — sans verdict, il n'est pas encore donné. */
  request: HitlRequest;
}

/**
 * Les 14 clés de la DEMANDE, disponibles à l'exécution.
 *
 * `Record<keyof HitlRequest, true>` rend l'exhaustivité une propriété de
 * COMPILATION : ajouter un champ à `HitlBinding` sans l'ajouter ici ne compile
 * pas (TS2739), et une clé qui n'existe pas dans le tuple ne compile pas non
 * plus. C'est le patron déjà employé par `FIELD_LABELS` dans `Approvals.tsx` —
 * les deux listes sont donc tenues par le même type, aucune ne peut dériver
 * seule. Ce que ce contrôle NE fait pas : garantir quoi que ce soit du côté
 * Rust ; cela reste la jambe `HITL-PARITY`.
 */
const REQUEST_FIELD_PRESENCE: Record<keyof HitlRequest, true> = {
  runId: true,
  planDigest: true,
  nodeInstanceId: true,
  effectId: true,
  capabilityRef: true,
  operation: true,
  destination: true,
  outgoingDigest: true,
  effectClass: true,
  maxCost: true,
  principal: true,
  nonce: true,
  seq: true,
  expiresAtMs: true,
};

const REQUEST_FIELDS = Object.keys(
  REQUEST_FIELD_PRESENCE,
) as (keyof HitlRequest)[];

/**
 * Une valeur porte-t-elle les 14 champs du tuple, tous en chaînes ?
 *
 * Écrit à la main, sans dépendance : `npm run size` a 40 octets de marge sur le
 * budget CSS et « 0 dep runtime nouvelle » est une décision Day-0. La convention
 * maison est la même ailleurs (`searchBrowse()`, `api/consent.ts`).
 */
function isValidRequest(value: unknown): value is HitlRequest {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return REQUEST_FIELDS.every((field) => typeof record[field] === "string");
}

function isValidApproval(value: unknown): value is PendingApproval {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return (
    typeof record.requestId === "string" &&
    record.requestId !== "" &&
    isValidRequest(record.request)
  );
}

/** Clé de seed lue au chargement du module — tests uniquement. */
const SEED_KEY = "__SBFB_HITL_QUEUE";

/**
 * Lit la fixture de test.
 *
 * BORNE ASSUMÉE, et non fermée (review r2 P2-22) : ce global est lu par le build
 * de PRODUCTION, celui-là même que l'e2e sert via `--web-root dist`.
 *
 * Le fermer derrière `import.meta.env.DEV` a été essayé et REJETÉ — mais PAS
 * pour la raison qu'une première rédaction donnait ici (« un gate vert sur une
 * page vide »), qui était fausse et le dit maintenant (review r4 R4-REG-1). La
 * mesure : sous garde DEV, QUATRE des cinq contrôles HITL passent au ROUGE — les
 * deux `toBeVisible()` sur une carte absente, la traversée clavier qui ne trouve
 * plus le bouton, ET le scan axe, dont le garde `toHaveCount(2)` précède
 * l'analyse ; seule « file vide » reste verte. Un gate rouge est le bon signal,
 * pas un gate aveugle. Le vrai arbitrage est donc « borne assumée CONTRE perte
 * de la couverture e2e seedée ».
 *
 * Troisième option, nommée pour qu'elle ne soit pas reconduite sans décision :
 * un flag de build positionné pour le seul `dist` e2e (`vite build --mode e2e`
 * + `define`), absent des releases. Écartée parce que l'e2e cesserait alors de
 * tester l'artefact de production, propriété que ce dépôt tient explicitement
 * (« un VRAI daemon, jamais un mock »). Détail complet : threat model §19.4,
 * borne 20 — les deux textes doivent rester d'accord.
 *
 * Ce que la borne coûte réellement, mesuré : rien de plus que l'accès console
 * dont elle dépend. Quiconque peut écrire `window.__SBFB_HITL_QUEUE` peut déjà
 * écrire dans le DOM et peindre la même fausse carte sans ce hook ; le seed
 * n'ajoute aucune capacité, il raccourcit un geste déjà disponible. Même
 * précédent assumé que `__SBFB_AUTH_TOKEN`, lu par `main.tsx` en production.
 *
 * Ce qui change à la RELIURE, et pourquoi ce hook doit alors disparaître : la
 * file viendra du daemon et une décision quittera le navigateur. Un global qui
 * pré-remplit la file deviendrait alors une porte d'entrée d'AUTORITÉ, pas un
 * raccourci d'affichage. À supprimer dans le commit qui câble la route, pas
 * après.
 */
/**
 * Filtre une valeur non fiable en file d'approbations.
 *
 * Rejette l'ITEM, jamais la file (correctif review r4 L1-R4-F03). Un cast
 * `as PendingApproval[]` laissait un `[{}]` traverser jusqu'au rendu, où la
 * déstructuration du tuple jetait et emportait toute la route /approvals via
 * `RouteErrorBoundary` — les approbations SAINES avec. Et à la reliure, un champ
 * manquant aurait rendu le texte `undefined` comme composante d'un tuple signé,
 * sans aucun signal.
 *
 * Exportée pour être testée DIRECTEMENT : `__seedHostQueueForTest` écrit l'état
 * interne sans passer par ici (c'est délibéré, cf. sa doc), donc un test qui
 * l'utiliserait pour vérifier ce filtre serait vacueux.
 */
export function sanitiseApprovalQueue(value: unknown): PendingApproval[] {
  if (!Array.isArray(value)) return [];
  return value.filter(isValidApproval);
}

function readSeededQueue(): PendingApproval[] {
  if (typeof window === "undefined") return [];
  return sanitiseApprovalQueue(
    (window as unknown as Record<string, unknown>)[SEED_KEY],
  );
}

let hostQueue: readonly PendingApproval[] = readSeededQueue();

/**
 * Snapshot courant de la file host-owned.
 *
 * `readonly` est load-bearing (correctif review r2, résidu P3) : un futur
 * producteur écrivant `getHostQueue().push(demande)` n'obtiendrait AUCUN
 * re-render — l'approbation serait invisible à l'opérateur. Avec ce type, ce
 * geste ne COMPILE pas : le producteur devra passer par une fonction de mutation
 * exportée qui réassigne la file, seule forme qui notifie React.
 */
export function getHostQueue(): readonly PendingApproval[] {
  return hostQueue;
}

/** Retire une approbation traitée (approuvée ou refusée) de la file. */
export function removeFromHostQueue(
  requestId: string,
): readonly PendingApproval[] {
  hostQueue = hostQueue.filter((a) => a.requestId !== requestId);
  return hostQueue;
}

/**
 * Injecte (ou vide) la file — TESTS UNIQUEMENT. Sans argument, remet l'état de
 * production : une file vide.
 *
 * Écrit l'état interne DIRECTEMENT, sans passer par
 * [`sanitiseApprovalQueue`] — délibérément, pour que les tests puissent
 * exercer la défense en profondeur du RENDU avec des valeurs qu'aucune entrée
 * réelle ne produirait. Conséquence à ne pas oublier : un test qui seede ici
 * ne prouve RIEN sur le filtre d'entrée ; celui-ci se teste par
 * `sanitiseApprovalQueue` directement.
 */
export function __seedHostQueueForTest(
  queue: readonly PendingApproval[] = [],
): void {
  hostQueue = queue;
}
