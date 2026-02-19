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

  return data.match_id;
}