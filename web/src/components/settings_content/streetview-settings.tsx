import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

// Zustand
import { StreetviewStateActions } from "@/store/useSettingsStore";
import { useStreetviewSettings } from "@/store/useSettingsStore";
import { useShallow } from "zustand/shallow";
import { settingsStateActions } from "@/store/useSettingsStore";

import { X } from "lucide-react";

const StreetviewSettings = () => {
  const { panning, streetNames, zooming, navigation } = useStreetviewSettings(
    useShallow((state) => ({
      panning: state.panningGesturesEnabled,
      streetNames: state.streetNamesEnabled,
      zooming: state.zoomGesturesEnabled,
      navigation: state.userNavigationEnabled,
    })),
  );

  return (
    <>
      <div className="flex flex-row justify-between items-center">
        <p className="text-xl font-semibold">Streetview</p>
        <X
          onClick={() => settingsStateActions.updateSettingsVisibility(false)}
        />
      </div>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="panningGesturesEnabled">Panning Gestures</Label>
        <Switch
          checked={panning}
          onCheckedChange={StreetviewStateActions.setPanningGesturesEnabled}
          id="panningGesturesEnabled"
        />
      </div>
      <Separator />

      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="streetNamesEnabled">Street Names</Label>
        <Switch
          checked={streetNames}
          onCheckedChange={StreetviewStateActions.setStreetNamesEnabled}
          id="streetNamesEnabled"
        />
      </div>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="zoomGesturesEnabled">Zoom Gestures</Label>
        <Switch
          checked={zooming}
          onCheckedChange={StreetviewStateActions.setZoomGesturesEnabled}
          id="zoomGesturesEnabled"
        />
      </div>
      <Separator />
      <div className="flex items-center justify-between space-x-2">
        <Label htmlFor="userNavigationEnabled">User Navigation</Label>
        <Switch
          checked={navigation}
          onCheckedChange={StreetviewStateActions.setUserNavigationEnabled}
          id="userNavigationEnabled"
        />
      </div>
    </>
  );
};

export default StreetviewSettings;
