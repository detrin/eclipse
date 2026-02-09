import type { GameState, Move, Hex, Occupant, Chain, ChainId, Player } from '../types/game';
import { hexEquals } from './hexUtils';

// Apply a move to the game state (client-side)
export function applyMove(state: GameState, move: Move): GameState {
  const newState = JSON.parse(JSON.stringify(state)) as GameState;

  newState.move_number += 1;
  newState.move_history.push(move);

  if ('MoveComet' in move) {
    const newPos = move.MoveComet;
    const currentPlayer = newState.current_turn;

    // Update comet position
    if (currentPlayer === 'Light') {
      const oldPos = newState.comet_light;
      // Remove from old position
      newState.occupied = newState.occupied.filter(
        entry => !hexEquals(entry.hex, oldPos)
      );
      // Add to new position
      newState.occupied.push({
        hex: newPos,
        occupant: { Comet: 'Light' }
      });
      newState.comet_light = newPos;
      newState.comet_light_last_moved = newState.move_number;
    } else {
      const oldPos = newState.comet_dark;
      // Remove from old position
      newState.occupied = newState.occupied.filter(
        entry => !hexEquals(entry.hex, oldPos)
      );
      // Add to new position
      newState.occupied.push({
        hex: newPos,
        occupant: { Comet: 'Dark' }
      });
      newState.comet_dark = newPos;
      newState.comet_dark_last_moved = newState.move_number;
    }
  } else if ('MoveSatellite' in move) {
    const { chain_id, old_pos, new_pos } = move.MoveSatellite;
    const currentPlayer = newState.current_turn;

    // Find the chain
    const chainIndex = newState.chains.findIndex(
      c => c.id === chain_id
    );

    if (chainIndex !== -1) {
      const chain = newState.chains[chainIndex];

      // Update chain positions
      const isMovingHead = hexEquals(chain.head, old_pos);
      if (isMovingHead) {
        chain.head = new_pos;
        chain.head_last_moved = newState.move_number;
      } else {
        chain.tail = new_pos;
        chain.tail_last_moved = newState.move_number;
      }

      // Remove from old position in occupied
      newState.occupied = newState.occupied.filter(
        entry => !hexEquals(entry.hex, old_pos)
      );

      // Add to new position in occupied
      newState.occupied.push({
        hex: new_pos,
        occupant: { Satellite: [chain_id, currentPlayer] }
      });
    }
  }

  // Switch turns
  newState.current_turn = newState.current_turn === 'Light' ? 'Dark' : 'Light';
  newState.status_messages = [];

  return newState;
}

// Get occupant at a specific hex
export function getOccupantAt(state: GameState, hex: Hex): Occupant | null {
  const entry = state.occupied.find(e => hexEquals(e.hex, hex));
  return entry ? entry.occupant : null;
}

// Get chain by ID
export function getChainById(state: GameState, chainId: ChainId): Chain | null {
  return state.chains.find(c => c.id === chainId) || null;
}

// Convert ChainId number to letter
export function chainIdToLetter(id: number): string {
  return String.fromCharCode(97 + id); // 97 is 'a'
}

// Check if a hex has a piece that belongs to the current player
export function isCurrentPlayerPiece(state: GameState, hex: Hex): boolean {
  const occupant = getOccupantAt(state, hex);
  if (!occupant) return false;

  if (occupant.Comet) {
    return occupant.Comet === state.current_turn;
  }
  if (occupant.Satellite) {
    return occupant.Satellite[1] === state.current_turn;
  }
  return false;
}
