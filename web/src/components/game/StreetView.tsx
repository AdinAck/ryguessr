"use client";
import { GoogleMap, StreetViewPanorama } from "@react-google-maps/api";
import { memo, useMemo } from "react";
import { StaticStreetview } from "./StaticStreetview";

// Zustand
import { useStreetviewSettings } from "@/store/useSettingsStore";

const Streetview = memo(function Streetview({ panoId }: { panoId?: string }) {
  const {
    panningGesturesEnabled,
    streetNamesEnabled,
    zoomGesturesEnabled,
    userNavigationEnabled,
  } = useStreetviewSettings();

  const streetViewOptions = useMemo(() => {
    return {
      // Custom controls
      clickToGo: userNavigationEnabled,
      linksControl: userNavigationEnabled,
      showRoadLabels: streetNamesEnabled,
      panControl: panningGesturesEnabled,
      scrollwheel: zoomGesturesEnabled,
      disableDoubleClickZoom: !zoomGesturesEnabled,
      zoomControl: zoomGesturesEnabled,

      // Default Controls
      visible: true,
      pano: panoId,
      addressControl: false,
      enableCloseButton: false,
      fullscreenControl: false,
      disableDefaultUI: true,
    };
  }, [
    panningGesturesEnabled,
    streetNamesEnabled,
    zoomGesturesEnabled,
    userNavigationEnabled,
    panoId,
  ]);

  return (
    <div className="h-full w-full">
      {panningGesturesEnabled ? (
        <GoogleMap
          zoom={10}
          mapContainerStyle={{ width: "100%", height: "100%" }}
        >
          <StreetViewPanorama options={streetViewOptions} />
        </GoogleMap>
      ) : (
        <StaticStreetview panoId={panoId} />
      )}
    </div>
  );
});

export default Streetview;
