import Coordinates from "./coordinate_type";
import RoundData from "./round-data";

interface MapOverlayProps {
  roundData?: RoundData,
  hasGuessed: boolean,
  handleGuess: (hasGuessed: boolean, selectedLocation: Coordinates) => void;
};

export default MapOverlayProps;
