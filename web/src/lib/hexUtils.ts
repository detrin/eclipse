import type { Hex } from '../types/game';

// Hex size for rendering
export const HEX_SIZE = 40;

// Convert axial coordinates to pixel coordinates for pointy-top hexagons
// Rotated 90° counter-clockwise so Light (human) is at bottom, Dark (bot) at top
export function hexToPixel(hex: Hex): { x: number; y: number } {
  // Calculate original coordinates
  const originalX = HEX_SIZE * (Math.sqrt(3) * hex.q + (Math.sqrt(3) / 2) * hex.r);
  const originalY = HEX_SIZE * ((3 / 2) * hex.r);

  // Rotate 90° counter-clockwise: (x, y) → (-y, x)
  const x = -originalY;
  const y = originalX;

  return { x, y };
}

// Check if hex is on the Eclipse board
export function isOnBoard(hex: Hex): boolean {
  const { q, r } = hex;
  switch (r) {
    case -3: return q >= -1 && q <= 4;
    case -2: return q >= -2 && q <= 4;
    case -1: return q >= -3 && q <= 4;
    case 0:  return q >= -3 && q <= 3;
    case 1:  return q >= -4 && q <= 3;
    case 2:  return q >= -4 && q <= 2;
    case 3:  return q >= -4 && q <= 1;
    default: return false;
  }
}

// Get all hexes on the board
export function getAllBoardHexes(): Hex[] {
  const hexes: Hex[] = [];

  const rowRanges: [number, number, number][] = [
    [-3, -1, 4],
    [-2, -2, 4],
    [-1, -3, 4],
    [0, -3, 3],
    [1, -4, 3],
    [2, -4, 2],
    [3, -4, 1],
  ];

  for (const [r, qMin, qMax] of rowRanges) {
    for (let q = qMin; q <= qMax; q++) {
      hexes.push({ q, r });
    }
  }

  return hexes;
}

// Check if two hexes are equal
export function hexEquals(a: Hex, b: Hex): boolean {
  return a.q === b.q && a.r === b.r;
}

// Get the six neighbors of a hex
export function getNeighbors(hex: Hex): Hex[] {
  const { q, r } = hex;
  return [
    { q: q + 1, r },       // East
    { q: q + 1, r: r - 1 }, // Northeast
    { q, r: r - 1 },       // Northwest
    { q: q - 1, r },       // West
    { q: q - 1, r: r + 1 }, // Southwest
    { q, r: r + 1 },       // Southeast
  ];
}

// Calculate distance between two hexes
export function hexDistance(a: Hex, b: Hex): number {
  const dq = Math.abs(a.q - b.q);
  const dr = Math.abs(a.r - b.r);
  const ds = Math.abs((a.q + a.r) - (b.q + b.r));
  return (dq + dr + ds) / 2;
}

// Points for drawing a flat-top hexagon (rotated 90° to match board orientation)
export function getHexagonPoints(size: number): string {
  const points: [number, number][] = [];
  for (let i = 0; i < 6; i++) {
    const angleDeg = 60 * i; // Start at 0° for flat-top (was -30° for pointy-top)
    const angleRad = (Math.PI / 180) * angleDeg;
    const x = size * Math.cos(angleRad);
    const y = size * Math.sin(angleRad);
    points.push([x, y]);
  }
  return points.map(([x, y]) => `${x},${y}`).join(' ');
}
