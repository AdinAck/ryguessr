"use client"

import { useEffect, useState } from "react"
import { GameView } from "@/components/game/GameView"


export default function Landing() {
  const [initialLocation, setInitialLocation] = useState(null)

  useEffect(() => {
    fetch("/api/random-location")
      .then((res) => res.json())
      .then(setInitialLocation)
  }, [])

  if (!initialLocation) return null

  return (
    <main className="m-0 p-0">
      <GameView initialLocation={initialLocation} />
    </main>
  )
}
