import Coordinates from "./coordinate_type";

interface MapOverlayProps {
  location?: Coordinates,
  hasGuessed: boolean,
  setHasGuessed: (hasGuessed: boolean) => void;
};

export default MapOverlayProps;
