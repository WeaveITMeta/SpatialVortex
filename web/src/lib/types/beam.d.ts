/**
 * Beam rendering and 3D visualization type definitions
 */

import type { ELPChannels } from './chat';

export interface BeamRenderParams {
  position: number;      // Flux position (0-9)
  ethos: number;        // Blue channel (0-9)
  logos: number;        // Green channel (0-9)
  pathos: number;       // Red channel (0-9)
  word?: string;        // Optional word label
  confidence?: number;  // Beam intensity (0-1)
}

export interface BeamParams {
  position: number;
  color: { r: number; g: number; b: number };
  intensity: number;
  curvature: number;
}

export interface BeamPath {
  start: number;        // Starting flux position
  end: number;          // Ending flux position
  points: Vector3[];    // Path waypoints
  curvature: number;    // Path curvature factor
}

export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

export interface FluxNode {
  position: number;     // Flux position (0-9)
  worldPos: Vector3;    // 3D world position
  isSacred: boolean;    // Is 3, 6, or 9
  color: { r: number; g: number; b: number };
  activity: number;     // Current activity level (0-1)
}

export interface IntersectionEffect {
  position: Vector3;
  color: { r: number; g: number; b: number };
  lifetime: number;
  maxLifetime: number;
  effectType: 'burst' | 'ripple' | 'ascension';
}

export interface BeamTrail {
  positions: Vector3[];
  colors: Array<{ r: number; g: number; b: number }>;
  maxLength: number;
}
