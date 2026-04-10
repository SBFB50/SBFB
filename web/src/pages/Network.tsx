/**
 * `/my-network` — stub Phase A. The live worker state view
 * arrives in Phase C once the Rust state_writer + coordinator
 * proxy are both wired and an end-to-end roundtrip test passes.
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function Network() {
  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Mon réseau</h1>
        <p className="text-sm text-muted-foreground">
          État live du worker nexus-grid qui tourne sur ta machine.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Phase C à venir</CardTitle>
          <CardDescription>
            Cette page affichera le GPU, les tâches traitées, les kudos
            par projet et la dernière tâche en temps réel.
          </CardDescription>
        </CardHeader>
        <CardContent className="text-xs text-muted-foreground">
          <p>
            Phase A a déjà wiré les pièces sous-jacentes : le worker
            Rust flush un snapshot JSON toutes les 5 s dans{" "}
            <code className="font-mono">~/.nexus-grid/worker/state.json</code>
            {" "}et le coordinateur expose{" "}
            <code className="font-mono">GET /worker-state</code> comme
            proxy. Phase C ajoute le rendu React avec React Query en
            polling 2 s.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
