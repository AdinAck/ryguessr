interface Coordinate {
  lat: number;
  lng: number;
}

function toRad(deg: number): number {
  return (deg * Math.PI) / 180;
}

function toDeg(rad: number): number {
  return (rad * 180) / Math.PI;
}

/**
 * Returns `n` evenly spaced points along the great-circle path
 * between two coordinates using spherical interpolation (slerp).
 */
export default function interpolateGreatCircle(
  start: Coordinate,
  end: Coordinate,
  n: number
): Coordinate[] {
  const lat1 = toRad(start.lat);
  const lng1 = toRad(start.lng);
  const lat2 = toRad(end.lat);
  const lng2 = toRad(end.lng);

  // Central angle between the two points (haversine formula)
  const d =
    2 *
    Math.asin(
      Math.sqrt(
        Math.sin((lat2 - lat1) / 2) ** 2 +
        Math.cos(lat1) * Math.cos(lat2) * Math.sin((lng2 - lng1) / 2) ** 2
      )
    );

  const points: Coordinate[] = [];

  for (let i = 0; i < n; i++) {
    const f = i / (n - 1); // fraction along the path [0, 1]

    if (d === 0) {
      // Points are identical
      points.push({ lat: start.lat, lng: start.lng });
      continue;
    }

    // Spherical linear interpolation (slerp)
    const a = Math.sin((1 - f) * d) / Math.sin(d);
    const b = Math.sin(f * d) / Math.sin(d);

    const x = a * Math.cos(lat1) * Math.cos(lng1) + b * Math.cos(lat2) * Math.cos(lng2);
    const y = a * Math.cos(lat1) * Math.sin(lng1) + b * Math.cos(lat2) * Math.sin(lng2);
    const z = a * Math.sin(lat1) + b * Math.sin(lat2);

    const lat = Math.atan2(z, Math.sqrt(x ** 2 + y ** 2));
    const lng = Math.atan2(y, x);

    points.push({ lat: toDeg(lat), lng: toDeg(lng) });
  }

  return points;
}
