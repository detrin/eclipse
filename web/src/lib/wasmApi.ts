import type { GameState, Move, ValidMovesResponse, BotResponse, VerifyResponse, WinningResponse, Player } from '../types/game';

// Import WASM module types
type WasmModule = {
  get_initial_state: () => string;
  get_valid_moves: (stateJson: string, player: string) => string;
  get_best_move: (stateJson: string, player: string, depth: number, weight: number) => string;
  verify_move: (stateJson: string, player: string, moveJson: string) => string;
  is_winning: (stateJson: string) => string;
};

let wasmModule: WasmModule | null = null;
let initPromise: Promise<void> | null = null;

/**
 * Initialize the WASM module
 * This should be called once at the start of your application
 */
export async function initWasm(): Promise<void> {
  if (wasmModule) {
    return; // Already initialized
  }

  if (initPromise) {
    return initPromise; // Already initializing
  }

  initPromise = (async () => {
    try {
      // Dynamic import of the WASM module from src/pkg
      const wasm = await import('../pkg/eclipse.js');

      // Initialize the WASM module
      await wasm.default();

      wasmModule = {
        get_initial_state: wasm.get_initial_state,
        get_valid_moves: wasm.get_valid_moves,
        get_best_move: wasm.get_best_move,
        verify_move: wasm.verify_move,
        is_winning: wasm.is_winning,
      };

      console.log('WASM module initialized successfully');
    } catch (error) {
      console.error('Failed to initialize WASM module:', error);
      throw new Error(`WASM initialization failed: ${error}`);
    }
  })();

  return initPromise;
}

/**
 * Ensure WASM is initialized before making calls
 */
function ensureInitialized(): WasmModule {
  if (!wasmModule) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }
  return wasmModule;
}

/**
 * Get the initial game state
 */
export async function getInitialState(): Promise<GameState> {
  const wasm = ensureInitialized();
  const resultJson = wasm.get_initial_state();
  return JSON.parse(resultJson);
}

/**
 * Get all valid moves for a player
 */
export async function getValidMoves(state: GameState, player: Player): Promise<ValidMovesResponse> {
  const wasm = ensureInitialized();
  const stateJson = JSON.stringify(state);
  const resultJson = wasm.get_valid_moves(stateJson, player);
  return JSON.parse(resultJson);
}

/**
 * Get the best move from the minimax bot
 */
export async function getBotMove(
  state: GameState,
  nextMove: Player,
  depth: number,
  weight: number
): Promise<BotResponse> {
  const wasm = ensureInitialized();
  const stateJson = JSON.stringify(state);
  const resultJson = wasm.get_best_move(stateJson, nextMove, depth, weight);
  return JSON.parse(resultJson);
}

/**
 * Verify if a move is legal
 */
export async function verifyMove(
  state: GameState,
  player: Player,
  move: Move
): Promise<VerifyResponse> {
  const wasm = ensureInitialized();
  const stateJson = JSON.stringify(state);
  const moveJson = JSON.stringify(move);
  const resultJson = wasm.verify_move(stateJson, player, moveJson);
  return JSON.parse(resultJson);
}

/**
 * Check if the current position is winning
 */
export async function isWinning(state: GameState): Promise<WinningResponse> {
  const wasm = ensureInitialized();
  const stateJson = JSON.stringify(state);
  const resultJson = wasm.is_winning(stateJson);
  return JSON.parse(resultJson);
}

/**
 * Health check (not applicable for WASM, returns success)
 */
export async function checkHealth(): Promise<{ status: string; service: string; version: string }> {
  return {
    status: 'ok',
    service: 'eclipse-wasm',
    version: '0.1.0',
  };
}
