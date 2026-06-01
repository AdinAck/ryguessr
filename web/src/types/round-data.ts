import Coordinates from "./coordinate_type";
import PlayerResults from "./player-results";

interface RoundData {
  real_location: Coordinates;
  player_results: Record<string, PlayerResults>;
}

export default RoundData;
