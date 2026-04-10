/**
 * `/project/:name` — stub Phase A. The rich 5-tab view (Overview,
 * Tasks, Kudos, Invites, Apps) arrives in Phase B.
 */

import { useParams } from "react-router-dom";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function ProjectDetail() {
  const { name } = useParams<{ name: string }>();

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Détail du projet</h1>
        <p className="text-sm text-muted-foreground">
          Projet : <span className="font-mono">{name ?? "—"}</span>
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Phase B à venir</CardTitle>
          <CardDescription>
            La vue détaillée (Overview, Tasks, Kudos, Invites, Apps)
            arrive dans le commit suivant.
          </CardDescription>
        </CardHeader>
        <CardContent className="text-xs text-muted-foreground">
          <p>
            Phase A livre uniquement le shell chrome et la route. Le
            rendu riche consomme directement le coordinateur actif et
            sera testé via Playwright contre un vrai coordinateur +
            vraies apps (hello, gov).
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
