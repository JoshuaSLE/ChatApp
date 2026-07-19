import api from "./client";

import type { LoginUser, LoginUserResponse } from "../types/users";

export const login = (data: LoginUser) =>
  api.post<LoginUserResponse>("/auth/login", data);

export const refresh = () => api.post<LoginUserResponse>("/auth/refresh");

export const logout = () => api.post("/auth/logout");
