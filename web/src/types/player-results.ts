import Coordinates from "./coordinate_type";
import PlayerData from "./player-data";

interface PlayerResults {
  player: PlayerData;
  round_score: number;
  distance: number;
  guess_location: Coordinates;
}

export default PlayerResults;
