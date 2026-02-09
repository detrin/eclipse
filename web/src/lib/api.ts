import type { GameState, Move, ValidMovesResponse, BotResponse, VerifyResponse, WinningResponse, Player } from '../types/game';

const API_BASE_URL = 'http://localhost:8080';

export async function getInitialState(): Promise<GameState> {
  const response = await fetch(`${API_BASE_URL}/initial_state`);
  if (!response.ok) {
    throw new Error(`Failed to fetch initial state: ${response.statusText}`);
  }
  return response.json();
}

export async function getValidMoves(state: GameState, player: Player): Promise<ValidMovesResponse> {
  const response = await fetch(`${API_BASE_URL}/valid_moves`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      state,
      player,
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch valid moves: ${response.statusText}`);
  }
  return response.json();
}

export async function getBotMove(
  state: GameState,
  nextMove: Player,
  depth: number,
  weight: number
): Promise<BotResponse> {
  const response = await fetch(`${API_BASE_URL}/bot`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      state,
      next_move: nextMove,
      depth,
      weight,
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to get bot move: ${response.statusText}`);
  }
  return response.json();
}

export async function verifyMove(
  state: GameState,
  player: Player,
  move: Move
): Promise<VerifyResponse> {
  const response = await fetch(`${API_BASE_URL}/verify`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      state,
      player,
      move,
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to verify move: ${response.statusText}`);
  }
  return response.json();
}

export async function isWinning(state: GameState): Promise<WinningResponse> {
  const response = await fetch(`${API_BASE_URL}/is_winning`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      state,
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to check winning state: ${response.statusText}`);
  }
  return response.json();
}

export async function checkHealth(): Promise<{ status: string; service: string; version: string }> {
  const response = await fetch(`${API_BASE_URL}/health`);
  if (!response.ok) {
    throw new Error(`Health check failed: ${response.statusText}`);
  }
  return response.json();
}
