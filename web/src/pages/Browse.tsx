/**
 * `/browse` — Sprint 6 stub per plan §2.4 (D4).
 *
 * DHT browse (pkarr) requires a local iroh node and thus a
 * shell-side daemon, which Sprint 5 explicitly does not build
 * to stay within the no-new-process scope cut.
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function Browse() {
  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Explorer</h1>
        <p className="text-sm text-muted-foreground">
          Découverte de projets publics via la DHT iroh-pkarr.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Arrive Sprint 6</CardTitle>
          <CardDescription>
            Cette page interrogera un sidecar local qui publie et
            résout des projets publics via la DHT. Le design est
            figé dans <code className="font-mono">sprint5_plan.md</code>{" "}
            §2.4 (décision D4) : pas de nouveau process en Sprint 5,
            le daemon <code className="font-mono">nexus-shell-daemon</code>{" "}
            arrive avec Sprint 6.
          </CardDescription>
        </CardHeader>
        <CardContent className="text-xs text-muted-foreground">
          <p>
            En attendant, utilise <strong>Mes projets</strong> pour
            voir les coordinateurs que tu as démarrés toi-même ou
            pour ajouter une URL connue via{" "}
            <strong>Ajouter un coordinateur</strong>.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
