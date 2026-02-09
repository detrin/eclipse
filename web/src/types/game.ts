export interface Hex {
  q: number;
  r: number;
}

export type Player = "Light" | "Dark";

export type ChainId = number;

export type ChainType = "Short" | "Long";

export interface Chain {
  id: ChainId;
  owner: Player;
  ctype: ChainType;
  head: Hex;
  tail: Hex;
  head_last_moved: number;
  tail_last_moved: number;
}

export interface Occupant {
  Comet?: Player;
  Satellite?: [ChainId, Player];
}

export interface OccupiedEntry {
  hex: Hex;
  occupant: Occupant;
}

export interface GameState {
  occupied: OccupiedEntry[];
  chains: Chain[];
  comet_light: Hex;
  comet_dark: Hex;
  current_turn: Player;
  status: "InProgress" | { Won: Player };
  move_history: Move[];
  status_messages: string[];
  move_number: number;
  comet_light_last_moved: number;
  comet_dark_last_moved: number;
}

export type Move =
  | { MoveComet: Hex }
  | {
      MoveSatellite: {
        chain_id: ChainId;
        old_pos: Hex;
        new_pos: Hex;
      };
    };

export interface ValidMovesResponse {
  success: boolean;
  valid_moves?: Move[];
  error?: string;
  move_count?: number;
}

export interface BotResponse {
  success: boolean;
  best_move?: Move;
  error?: string;
  score?: number;
  legal_moves_count?: number;
}

export interface VerifyResponse {
  success: boolean;
  is_legal?: boolean;
  error?: string;
  reason?: string;
  move_verified?: Move;
}

export interface WinningResponse {
  success: boolean;
  is_winning?: boolean;
  winner?: Player;
  error?: string;
}
