# Protocole et data flow

## Flux app SBFB

```text
repo public
  -> SBFB.json
  -> provenance
  -> archive zip
  -> iroh-blobs
  -> ProjectAnnouncement
  -> BrowseEntry
  -> iframe sandboxee
```

## Flux runtime

```text
App iframe
  -> postMessage task_submit
  -> shell React
  -> daemon local
  -> TaskEntry signee dans iroh-docs
  -> Worker lit la task
  -> ClaimEntry signee
  -> execution locale GPU/CPU
  -> ResultEntry signee
  -> validation
  -> kudos / events / retour UI
```

## Bridge SBFB

Methodes visibles aujourd'hui:

- `task_submit`
- `storage_get`
- `storage_set`
- `pii_redact`

Le bridge est volontairement petit: l'app en iframe ne parle pas directement au
filesystem, au reseau ou au broker. Elle demande au shell, qui demande au
daemon.

## Ce que cela veut dire pour Babel

Babel n'a pas besoin de casser l'architecture. Il peut devenir une app SBFB:

```text
Babel UI -> Babel Shelf offline -> task_submit traduction/indexation
```

Puis plus tard:

```text
Babel Shelf -> synchronisation liseuse -> corpus local
```

La liseuse ne doit pas devenir une dependance Amazon/Kobo. Elle doit recevoir
des contenus Babel depuis un chemin libre: USB, Wi-Fi local, serveur local,
noeud SBFB leger, ou firmware Babel.
