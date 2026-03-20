"use client"
import { useEffect, useState, useCallback, useRef, useMemo, memo, Fragment } from 'react';
import { GoogleMap, Marker, Polyline } from '@react-google-maps/api';
import { Button } from '../ui/button';
import interpolateGreatCircle from '@/lib/haversine';

// Coordinates type
import MapOverlayProps from '@/types/map_overlay_props';
import Coordinates from '@/types/coordinate_type';
import PlayerResults from '@/types/player-results';

export const MapOverlay = memo(({ hasContinued, roundData, handleGuess, hasGuessed, handleContinue, shownScoreboard, handleScoreboard }: MapOverlayProps) => {
  const [selectedLocation, setSelectedLocation] = useState<Coordinates | null>(null);

  const { userIconOptions, poly_color } = useMemo(() => {

    const svgString = `<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="black" stroke="#FFFFFF" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-map-pin-icon lucide-map-pin"><path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"/><circle cx="12" cy="10" r="3"/></svg>`

    const customIconUrl = `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(svgString)}`;
    return {
      userIconOptions: { width: 30, height: 30, url: customIconUrl, anchor: new google.maps.Point(16, 30), scaledSize: new window.google.maps.Size(32, 32) },
      poly_color: "#FF0000",
    };
  }, [])

  const { locationIconOptions, mapContainerStyle } = useMemo(() => {
    return (
      {
        locationIconOptions: { width: 30, height: 30, url: "/svg/flag.svg", anchor: new google.maps.Point(6, 30) },
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

  const createPlayerIcon = useCallback((color: string) => {
    const svgString = `
      <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="${color}" stroke="#FFFFFF" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-map-pin-icon lucide-map-pin">
        <path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"/>
        <circle cx="12" cy="10" r="3"/>
      </svg>
    `;

    return {
      url: `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(svgString)}`,
      anchor: window.google ? new window.google.maps.Point(15, 20) : undefined,
      scaledSize: window.google ? new window.google.maps.Size(32, 32) : undefined,
    };
  }, []);


  return (
    <div className="flex gap-2 flex-col gap-2 h-full w-full">
      <div className={`${roundData?.player_results && shownScoreboard ? 'blur-sm pointer-events-none' : undefined} flex-grow w-full rounded-lg overflow-hidden border-2 shadow-xl relative z-0`}>

        <GoogleMap
          mapContainerStyle={mapContainerStyle}
          onLoad={onLoad}
          options={mapOptions}
          onClick={!hasGuessed ? onMapClick : undefined}
        >
          {selectedLocation && (
            <Marker
              position={selectedLocation}
              icon={userIconOptions}
              zIndex={50}
            />

          )}
          {hasGuessed && selectedLocation && roundData && (
            <>
              <Marker
                position={{ lat: roundData.real_location.lat, lng: roundData.real_location.lng }}
                icon={locationIconOptions}
              />
              <Polyline options={{
                strokeColor: poly_color,
                strokeOpacity: 0,
                zIndex: 10,
                icons: [
                  {
                    icon: {
                      path: "M 0,0 0,-2",
                      strokeOpacity: 1,
                      scale: 3,
                    },
                    offset: "12px",
                    repeat: "20px",
                  },
                ],
              }}
                path={interpolateGreatCircle(selectedLocation, roundData.real_location, 100)} />
              {Object.values(roundData.player_results).map((entry: PlayerResults) => {
                const isCurrentUser =
                  Math.abs(entry.guess_location.lat - selectedLocation.lat) < 0.00001 &&
                  Math.abs(entry.guess_location.lng - selectedLocation.lng) < 0.00001;

                if (isCurrentUser) return null;

                return (
                  <Fragment key={entry.color}>

                    <Marker
                      position={entry.guess_location}
                      icon={createPlayerIcon(entry.color)}
                    />

                    <Polyline
                      path={interpolateGreatCircle(entry.guess_location, roundData.real_location, 100)}
                      options={{
                        strokeColor: entry.color,
                        strokeOpacity: 0,
                        icons: [
                          {
                            icon: {
                              path: "M 0,0 0,-2",
                              strokeOpacity: 1,
                              scale: 3,
                              strokeColor: entry.color,
                            },
                            offset: "12px",
                            repeat: "20px",
                          },
                        ],
                      }}
                    />
                  </Fragment>
                );
              })}
            </>
          )}


        </GoogleMap>
      </div>
      <div className='w-full z-10 rounded-lg bg-background transition-transform duration-150 ease-in-out active:scale-95'>
        <Button
          className={"w-full bg-background text-white tracking-widest shadow-lg"}
          size={"default"}
          variant={"outline"}
          onClick={
            hasContinued ? undefined
              : shownScoreboard ? handleContinue
                : hasGuessed ? handleScoreboard
                  : selectedLocation ? handleMapPanOnGuess
                    : undefined
          }
        > {hasContinued ? "Waiting for Players..."
          : shownScoreboard ? "Continue"
            : hasGuessed && roundData ? "Show Scoreboard"
              : hasGuessed && !roundData ? "Waiting for Players..."
                : "Guess"
          } </Button>
      </div>
    </div>
  )
});


{/* : shownScoreboard ? "Continue" */ }
