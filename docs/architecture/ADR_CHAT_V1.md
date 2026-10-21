# ADR — Messagerie v1 : chat chiffré de bout en bout sur iroh

- **Statut** : accepté (PO, 2026-10-21)
- **Remplace** : rien. **Amende** la doctrine « pas de groupes privés » (2026-07-29)
  pour la seule messagerie E2E.
- **Relation** : le plan « Messagerie anonyme » (Tor + serveurs de files `.onion`,
  2026-10-19) devient la **v2** ; il n'est pas abandonné.

## Contexte

Le réseau est déployable depuis la v1.1.0 (relais et pkarr auto-hébergés, ancre,
réseau public par défaut). Le PO veut une app de discussion utilisable rapidement,
basée sur iroh, chiffrée, publiée comme une app SBFB, en réutilisant de l'open
source. Le plan anonyme (≈ 6-7 mois) ne répond pas à cette échéance.

## Décisions

1. **Pas d'anonymat réseau en v1.** Le contenu est chiffré de bout en bout ;
   l'acheminement passe par iroh (gossip, relais SBFB). Les métadonnées (qui parle à
   qui, quand, depuis quelle IP) ne sont pas protégées et l'app le dit.
2. **1:1 et groupes dès la v1** (taille max 50).
3. **Le chiffrement tourne dans le daemon Rust**, pas dans l'iframe. Bibliothèque :
   **vodozemac** (Olm pour l'échange de clés par paire, Megolm pour les sessions de
   groupe ; Apache-2.0 ; auditée par Least Authority). Les clés ne sont jamais
   exposées au JavaScript. Renverse « crypto dans l'app (WASM) » du 2026-10-19.
4. **Transport** : un document iroh-docs par conversation, entrées = ciphertext
   opaque signé. Retrait d'un membre = rotation des sessions **et** nouveau document
   (un ticket d'écriture iroh-docs n'est pas révocable).
5. **Hors ligne** : l'ancre peut garder le chiffré, **en opt-in**, via un ticket de
   **lecture** (elle ne peut pas écrire), TTL 21 jours, quotas.

## Amendement de la doctrine « pas de groupes privés »

La doctrine du 2026-07-29 ferme les **espaces privés persistants** (appartenance
durable, « dedans/dehors ») parce qu'un espace opaque est un angle mort sans
attribution dans un réseau sans modération centrale.

Elle est **levée pour la messagerie E2E uniquement**, avec ces bornes :

- un groupe de discussion n'est **ni curé, ni annoncé, ni indexé** : il n'apparaît
  dans aucun Browse, aucune liste de curateur, aucun annuaire ;
- il ne peut pas **publier** au nom du groupe : il n'existe pas d'identité de
  groupe qui signerait un contenu public ;
- taille bornée (50), ce qui le garde du côté « conversation », pas « plateforme » ;
- tout le reste du réseau (artefacts, workflows, stockage d'app) reste soumis à la
  doctrine : un artefact est publié ou local, sans liste de lecteurs persistante.

Le test de la doctrine reste valable ailleurs : une liste qui survit à la donnée
qu'elle protège est un espace persistant.

## Alternatives écartées

| Option | Raison |
|---|---|
| openmls (MLS, RFC 9420) | exige un ordre total des Commits, donc un séquenceur, en P2P |
| p2panda-encryption / p2panda-net | non audité ; p2panda-net veut posséder l'endpoint iroh. Piste v2 (meilleure sécurité post-compromission) |
| Keyhive / BeeKEM | pré-alpha |
| Crypto dans l'iframe (WASM) | spike lourd (aléa, WASM base64, coffre côté app) ; aucune clé ne doit vivre dans le JS de l'éditeur |
| Delta Chat core | iroh 0.35 ; reste une **référence de design** (API realtime webxdc) |

## Conséquences

- Nouvelle crate `nexus-messaging-core`, module daemon `msg_api`, verbes `msg_*`
  (classe sensible, consentement à l'installation), ALPN `sbfb/mailbox/0`,
  app `examples/sbfb-chat`.
- `docs/security/THREAT_MODEL.md` §20 décrit la surface.
- Megolm offre une sécurité post-compromission faible : sessions tournées
  périodiquement. Statut « expérimental » tant qu'aucun audit externe du protocole
  SBFB au-dessus de vodozemac n'a eu lieu.

## Preuves (spike A, GO)

- `crates/nexus-messaging-core/tests/spike_olm_megolm.rs` : groupe à 3, falsification
  rejetée, membre retiré qui ne lit plus après rotation, 1:1 = groupe à 2, compte
  scellé par clé.
- `crates/nexus-core-rs/tests/chat_docs_spike.rs` : synchronisation entre membres,
  réplique en lecture qui ne peut pas écrire, membre arrivé plus tard qui récupère
  tout l'historique depuis la seule boîte aux lettres (mutation contrôlée : sans la
  synchronisation vers la boîte aux lettres, le test échoue).
