"use client"
import { useEffect, useState, useCallback, useRef, useMemo, memo } from 'react';
import { GoogleMap, Marker, Polyline } from '@react-google-maps/api';
import { Button } from '../ui/button';
import interpolateGreatCircle from '@/lib/haversine';

// Coordinates type
import MapOverlayProps from '@/types/map_overlay_props';
import Coordinates from '@/types/coordinate_type';

export const MapOverlay = memo(({ hasContinued, roundData, handleGuess, hasGuessed, handleContinue }: MapOverlayProps) => {
  const [selectedLocation, setSelectedLocation] = useState<Coordinates | null>(null);

  const { defaultPosition, locationIconOptions, userIconOptions, mapContainerStyle } = useMemo(() => {
    return (
      {
        defaultPosition: { lat: 35.6586, lng: 139.7454 },
        locationIconOptions: { width: 30, height: 30, url: "/svg/flag.svg", anchor: new google.maps.Point(6, 30) },
        userIconOptions: { width: 30, height: 30, url: "/svg/user-map-pin.svg", anchor: new google.maps.Point(15, 24) },
        mapContainerStyle: { width: "100%", height: "100%" },
      }
    );

  }, [])

  const mapRef = useRef<google.maps.Map | null>(null);

  const mapOptions = useMemo(() => {
    return { draggableCursor: hasGuessed ? "move" : "crosshair", disableDefaultUI: true, clickableIcons: false, gestureHandling: "greedy", draggingCursor: "move", minZoom: 2 }
  }, [hasGuessed]);


  const onMapClick = useCallback((e: google.maps.MapMouseEvent) => {
    if (e.latLng) {
      setSelectedLocation({ lat: e.latLng.lat(), lng: e.latLng.lng() })
    };
  }, []);

  const onLoad = useCallback((map: google.maps.Map) => {
    mapRef.current = map;

    map.setCenter({ lat: 20, lng: 0 });
    map.setZoom(2);
  }, []);

  const handleMapPanOnGuess = () => {
    if (!selectedLocation) return;
    handleGuess(true, selectedLocation);
  }

  useEffect(() => {
    if (hasGuessed && roundData?.real_location && selectedLocation && mapRef.current) {
      const bounds = new window.google.maps.LatLngBounds();
      bounds.extend(roundData.real_location);
      bounds.extend(selectedLocation);
      mapRef.current.fitBounds(bounds, 50);
    };
  }, [roundData, selectedLocation, hasGuessed])

  return (
    <div className="flex gap-2 flex-col gap-2 h-full w-full">
      <div className='flex-grow w-full rounded-lg overflow-hidden border-2 shadow-xl relative z-0'>

        <GoogleMap
          mapContainerStyle={mapContainerStyle}
          onLoad={onLoad}
          options={mapOptions}
          onClick={!hasGuessed ? onMapClick : undefined}
        >
          {selectedLocation && (<Marker
            position={selectedLocation}
            icon={userIconOptions}

          />)}
          {hasGuessed && selectedLocation && roundData && (
            <>
              <Marker
                icon={locationIconOptions}
                position={selectedLocation ? { lat: roundData.real_location.lat, lng: roundData.real_location.lng } : defaultPosition}
              />
              ) &&
              (
              <Polyline options={{
                strokeColor: "#FF0000",
                strokeOpacity: 0,
                icons: [
                  {
                    icon: {
                      path: "M 0,-1 0,1",
                      strokeOpacity: 1,
                      scale: 4,
                    },
                    offset: "0",
                    repeat: "20px",
                  },
                ],
              }}
                path={interpolateGreatCircle(selectedLocation, roundData.real_location, 100)} />
            </>
          )
          }
        </GoogleMap>
      </div>
      <div className='w-full z-10'>
        <Button
          className={"w-full tracking-widest shadow-lg transition-transform duration-150 ease-in-out active:scale-95"}
          size={"default"}
          variant={"default"}
          onClick={
            hasContinued ? undefined
              // : shownScoreboard ? handleContinue
              : hasGuessed ? handleContinue
                : selectedLocation ? handleMapPanOnGuess
                  : undefined
          }
        > {hasContinued ? "Waiting for Players..."
          : hasGuessed ? "Continue"
            : "Guess"
          } </Button>
      </div>
    </div>
  );
});


{/* : shownScoreboard ? "Continue" */ }
