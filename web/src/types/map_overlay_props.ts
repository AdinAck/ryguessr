import Coordinates from "./coordinate_type";
import RoundData from "./round-data";

interface MapOverlayProps {
  roundData?: RoundData;
  hasGuessed: boolean;
  hasContinued: boolean;
  shownScoreboard: boolean;
  mapExpanded: boolean;
  handleSetMapExpanded: (mapExpanded: boolean) => void;
  handleGuess: (hasGuessed: boolean, selectedLocation: Coordinates) => void;
  handleScoreboard: () => void;
  handleContinue: () => void;
}

export default MapOverlayProps;
