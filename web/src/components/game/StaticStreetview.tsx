import Image from "next/image";

// Zustand
import { useAPIKEY } from "@/store/useSettingsStore";

export const StaticStreetview = ({ panoId }: { panoId?: string }) => {
  const google_maps_api_key = useAPIKEY((state) => state.google_maps_api_key);

  const imageUrl = `https://maps.googleapis.com/maps/api/streetview?size=960x540&pano=${panoId}&pitch=0&fov=90&key=${google_maps_api_key}`;

  return (
    <div className="relative h-full w-full bg-black">
      <Image
        src={imageUrl}
        alt="Street location"
        fill
        className="object-cover"
        draggable={false}
        unoptimized
      />
    </div>
  );
};
