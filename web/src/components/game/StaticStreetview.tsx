import Image from "next/image";

export const StaticStreetview = ({ panoId }: { panoId?: string }) => {
  const imageUrl = `https://maps.googleapis.com/maps/api/streetview?size=960x540&pano=${panoId}&pitch=0&fov=90&key=${process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY}`;

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
