const BASE_URL = "http://localhost:8000";

export async function apiPost(path: string, data: any) {
  const response = await fetch(BASE_URL + path, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    credentials: "include", 
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    throw new Error("API Fehler");
  }

  return response.json();
}