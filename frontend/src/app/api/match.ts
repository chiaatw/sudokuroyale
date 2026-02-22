const BASE_URL = "/api";

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

export async function startMatch(matchId: string): Promise<boolean> {
  const res = await fetch(`${BASE_URL}/match/start`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify({ matchId }),
  });

  if (!res.ok) {
    return false;
  }

  const data = await res.json();
  return data.ok === true;
}

export async function getMatch(matchId: string) {
  const res = await fetch(`${BASE_URL}/match/${matchId}`, {
    credentials: "include",
  });

  if (!res.ok) {
    throw new Error("Match laden fehlgeschlagen");
  }

  return res.json();
}