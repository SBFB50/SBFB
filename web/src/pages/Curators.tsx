/**
 * `/curators` — Sprint 6 stub per plan §2.4 (D4) and §2 of the
 * phoenix plan (20 frozen decisions, item #6 "curator lists via
 * iroh-blobs + iroh-gossip").
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function Curators() {
  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Curators</h1>
        <p className="text-sm text-muted-foreground">
          Listes curées de projets publics signées Ed25519.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Arrive Sprint 6</CardTitle>
          <CardDescription>
            Les curator lists arrivent avec le flux gossip iroh-blobs
            décidé dans le plan phoenix (décision figée #6). Sprint 5
            livre uniquement le shell et les pages qui consomment des
            données locales — le flux de découverte réseau suit au
            sprint suivant.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-xs text-muted-foreground">
          <p>Cette page affichera :</p>
          <ul className="ml-4 list-disc space-y-1">
            <li>Les curator lists auxquelles tu es abonné</li>
            <li>Un bouton « Gérer mes curators » pour ajouter ou retirer</li>
            <li>
              Chaque liste avec sa signature Ed25519 vérifiée et son
              contenu (projets featured)
            </li>
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}
