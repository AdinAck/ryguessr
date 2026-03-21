import Coordinates from "./coordinate_type"

interface PlayerResults {
  last_score: number,
  cum_score: number,
  distance: number,
  guess_location: Coordinates,
  color: string,
};

export default PlayerResults;
