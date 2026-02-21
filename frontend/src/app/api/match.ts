const BASE_URL = "http://localhost:8000";

export async function createMatch(): Promise<string> {
  const res = await fetch(`${BASE_URL}/match/create`, {
    method: "POST",
    credentials: "include", 
  });

  if (!res.ok) {
    throw new Error("Match erstellen fehlgeschlagen");
  }

  const data = await res.json();

  return data.matchId;
}

export async function joinMatch(matchId: string): Promise<boolean> {
  const res = await fetch(`${BASE_URL}/match/join`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    credentials: "include",
    body: JSON.stringify({ matchId }),
  });

  if (!res.ok) {
    throw new Error("Match beitreten fehlgeschlagen");
  }

  const data = await res.json();

  return data.ok;
}