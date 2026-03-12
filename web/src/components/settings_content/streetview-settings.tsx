import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

// Zustand
import { StreetviewStateActions } from "@/store/useSettingsStore";
import { useStreetviewSettings } from "@/store/useSettingsStore";
import { useShallow } from "zustand/shallow";

const StreetviewSettings = () => {
  const { panning, streetNames, zooming, navigation } = useStreetviewSettings(
    useShallow((state) => ({
      panning: state.panningGesturesEnabled,
      streetNames: state.streetNamesEnabled,
      zooming: state.zoomGesturesEnabled,
      navigation: state.userNavigationEnabled,
    }))
  );

  return (
    <>
      <p className="text-xl font-semibold">Streetview</p>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="panningGesturesEnabled">Enable Panning Gestures</Label>
        <Switch checked={panning} onCheckedChange={StreetviewStateActions.setPanningGesturesEnabled} id="panningGesturesEnabled" />
      </div>
      <Separator />

      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="streetNamesEnabled">Enable Street Names</Label>
        <Switch checked={streetNames} onCheckedChange={StreetviewStateActions.setStreetNamesEnabled} id="streetNamesEnabled" />
      </div>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="zoomGesturesEnabled">Enable Zoom Gestures</Label>
        <Switch checked={zooming} onCheckedChange={StreetviewStateActions.setZoomGesturesEnabled} id="zoomGesturesEnabled" />
      </div>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="userNavigationEnabled">Enable User Navigation</Label>
        <Switch checked={navigation} onCheckedChange={StreetviewStateActions.setUserNavigationEnabled} id="userNavigationEnabled" />
      </div>
    </>
  );
}

export default StreetviewSettings;
