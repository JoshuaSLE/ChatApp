import api from "./client";

import type {
  RegisterUser,
  SearchUser,
  SearchUserResponse,
  StatusUser,
  StatusUserResponse,
  UpdateUser,
  UpdateUserResponse,
  UserResponse,
} from "../types/users";

export const register = (data: RegisterUser) =>
  api.post<UserResponse>("/user/register", data);

export const update = (data: UpdateUser) =>
  api.patch<UpdateUserResponse>("/user/update", data);

export const remove = () => api.delete("/user/delete");

export const me = () => api.get("/user/me");

export const search = (params: SearchUser) =>
  api.get<SearchUserResponse[]>("/user/search", { params });

export const status = (params: StatusUser) =>
  api.get<StatusUserResponse>("/user/status", { params });
