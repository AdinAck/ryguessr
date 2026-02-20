import { Location } from "./coordinate_type";

export type MapOverlayProps = {
  location?: Location,
  hasGuessed: boolean,
  setHasGuessed: (hasGuessed: boolean) => void;
};
