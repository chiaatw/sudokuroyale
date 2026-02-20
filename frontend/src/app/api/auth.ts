import { apiPost, apiGet } from "./client";

export type RegisterRequest = {
  username: string;
  email: string;
  password: string;
};

export type LoginRequest = {
  username: string;
  password: string;
};

export async function register(req: RegisterRequest) {
  return apiPost<{ message: string }>("/register", req);
}

export async function login(req: LoginRequest) {
  return apiPost<{ message: string }>("/login", req);
}

export async function me() {
  return apiGet<{ username: string; userId: string }>("/me");
}