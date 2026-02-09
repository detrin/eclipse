/* tslint:disable */
/* eslint-disable */

/**
 * Get the best move for a given game state using the minimax bot
 *
 * # Arguments
 * * `state_json` - JSON string representation of the game state
 * * `player` - The player to move next ("light" or "dark")
 * * `depth` - Search depth for minimax algorithm (1-7)
 * * `weight` - Evaluation weight multiplier
 *
 * # Returns
 * JSON string containing the best move or an error message
 *
 * # Example (JavaScript)
 * ```javascript
 * const response = get_best_move(stateJson, "light", 3, 1.0);
 * const result = JSON.parse(response);
 * if (result.success) {
 *     console.log("Best move:", result.best_move);
 * }
 * ```
 */
export function get_best_move(state_json: string, player: string, depth: number, weight: number): string;

/**
 * Get the initial game state as JSON
 *
 * # Returns
 * JSON string containing the initial game state
 *
 * # Example (JavaScript)
 * ```javascript
 * import init, { get_initial_state } from './eclipse_wasm.js';
 *
 * await init();
 * const stateJson = get_initial_state();
 * const state = JSON.parse(stateJson);
 * ```
 */
export function get_initial_state(): string;

/**
 * Get all valid moves for a player in a given game state
 *
 * # Arguments
 * * `state_json` - JSON string representation of the game state
 * * `player` - The player whose moves to return ("light" or "dark")
 *
 * # Returns
 * JSON string containing all valid moves or an error message
 *
 * # Example (JavaScript)
 * ```javascript
 * const response = get_valid_moves(stateJson, "light");
 * const result = JSON.parse(response);
 * if (result.success) {
 *     console.log("Valid moves:", result.valid_moves);
 * }
 * ```
 */
export function get_valid_moves(state_json: string, player: string): string;

/**
 * Initialize WASM module - sets up panic hook for better error messages
 */
export function init(): void;

/**
 * Check if the current position is a win for the player who just moved
 *
 * # Arguments
 * * `state_json` - JSON string representation of the game state
 *
 * # Returns
 * JSON string indicating whether the position is winning and who won
 *
 * # Example (JavaScript)
 * ```javascript
 * const response = is_winning(stateJson);
 * const result = JSON.parse(response);
 * if (result.is_winning) {
 *     console.log("Winner:", result.winner);
 * }
 * ```
 */
export function is_winning(state_json: string): string;

/**
 * Verify if a move is legal for a given game state
 *
 * # Arguments
 * * `state_json` - JSON string representation of the game state
 * * `player` - The player making the move ("light" or "dark")
 * * `move_json` - JSON string representation of the move to verify
 *
 * # Returns
 * JSON string indicating whether the move is legal and why
 *
 * # Example (JavaScript)
 * ```javascript
 * const response = verify_move(stateJson, "light", moveJson);
 * const result = JSON.parse(response);
 * if (result.is_legal) {
 *     console.log("Move is legal!");
 * } else {
 *     console.log("Move is illegal:", result.reason);
 * }
 * ```
 */
export function verify_move(state_json: string, player: string, move_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_best_move: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly get_initial_state: () => [number, number];
    readonly get_valid_moves: (a: number, b: number, c: number, d: number) => [number, number];
    readonly is_winning: (a: number, b: number) => [number, number];
    readonly verify_move: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly init: () => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
