declare module 'd3-parliament-chart' {
  export interface ParliamentPoint {
    x: number;
    y: number;
  }

  export interface ParliamentOptions {
    sections?: number;
    sectionGap?: number;
    seatRadius?: number;
    rowHeight?: number;
  }

  export function getParliamentPoints(
    totalPoints: number,
    options: ParliamentOptions,
    graphicWidth: number,
  ): ParliamentPoint[];

  export function parliamentChart(): any;
}
