const BASE_URL = "http://localhost:8000";

export async function apiGet<T>(path: string): Promise<T> {
  const response = await fetch(BASE_URL + path, {
    credentials: "include",
  });

  if (!response.ok) {
    throw new Error("API Fehler");
  }

  return response.json();
}


export async function apiPost<T>(path: string, data: any): Promise<T> {
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
