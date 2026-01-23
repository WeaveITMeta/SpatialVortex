/**
 * WASM module type definitions for Bevy integration
 */

import type { BeamRenderParams } from './beam';

export interface WASMModule {
  default: (config?: { module_or_path?: string }) => Promise<void>;
  render_beam: (params: BeamRenderParams) => void;
  init_flux_matrix: () => void;
  update_camera: (position: { x: number; y: number; z: number }) => void;
  trigger_sacred_effect: (position: number) => void;
}

export interface WASMConfig {
  module_or_path?: string;
  memory?: WebAssembly.Memory;
  canvas?: HTMLCanvasElement;
}

export interface WASMExports {
  memory: WebAssembly.Memory;
  render_frame: () => void;
  handle_input: (eventType: number, x: number, y: number) => void;
}

declare global {
  interface Window {
    renderBeam?: (params: BeamRenderParams) => void;
    initFluxMatrix?: () => void;
    updateCamera?: (position: { x: number; y: number; z: number }) => void;
    triggerSacredEffect?: (position: number) => void;
  }
}

export {};
